use std::f32::consts::TAU;

// ADSR constants
const ATTACK_SECS:    f32 = 0.008;
const DECAY_SECS:     f32 = 0.100;
const SUSTAIN_LEVEL:  f32 = 0.70;
const RELEASE_SECS:   f32 = 0.300;
// Emergency fallback: if neither a NoteOff command nor the poll-based synthetic
// release fires (e.g. app hung), cap sustain here. Normal release is handled
// by the input layer's RELEASE_TIMEOUT (~80ms), so this should rarely trigger.
const MAX_SUSTAIN_SECS: f32 = 10.0;

// Additive harmonics: (harmonic_number, amplitude)
const HARMONICS: [(f32, f32); 4] = [
    (1.0, 1.000),
    (2.0, 0.500),
    (3.0, 0.250),
    (4.0, 0.125),
];

#[derive(Debug, Clone, Copy, PartialEq)]
enum EnvPhase { Attack, Decay, Sustain, Release, Off }

/// A single active piano note with its own ADSR envelope.
/// Lives entirely inside the audio callback — no locks, no allocation after creation.
pub struct Voice {
    pub midi: u8,
    freq: f32,          // slightly detuned frequency
    velocity: f32,      // 0.0..=1.0
    time: f32,          // seconds since note-on
    dt: f32,            // 1.0 / sample_rate
    env_phase: EnvPhase,
    env_level: f32,
    release_from: f32,  // envelope level at the moment of key release
    release_at: f32,    // time.value when release began
}

impl Voice {
    pub fn new(midi: u8, freq: f32, velocity: u8, sample_rate: f32) -> Self {
        // Deterministic per-note detuning ±2 cents. Avoids the "digital" perfect-sine feel.
        let cents = ((midi as i32 * 7 + 3) % 5 - 2) as f32; // -2..+2
        let detune = 2f32.powf(cents / 1200.0);
        Voice {
            midi,
            freq: freq * detune,
            velocity: velocity as f32 / 127.0,
            time: 0.0,
            dt: 1.0 / sample_rate,
            env_phase: EnvPhase::Attack,
            env_level: 0.0,
            release_from: 0.0,
            release_at: 0.0,
        }
    }

    /// Begin the release phase (key up).
    pub fn release(&mut self) {
        if !matches!(self.env_phase, EnvPhase::Release | EnvPhase::Off) {
            self.release_from = self.env_level;
            self.release_at = self.time;
            self.env_phase = EnvPhase::Release;
        }
    }

    pub fn is_active(&self) -> bool {
        self.env_phase != EnvPhase::Off
    }

    /// Generate the next audio sample. Called 44100× per second on the audio thread.
    pub fn next_sample(&mut self) -> f32 {
        // Advance envelope
        self.env_level = match self.env_phase {
            EnvPhase::Attack => {
                let level = self.time / ATTACK_SECS;
                if level >= 1.0 {
                    self.env_phase = EnvPhase::Decay;
                    1.0
                } else {
                    level
                }
            }
            EnvPhase::Decay => {
                let t = self.time - ATTACK_SECS;
                let level = 1.0 - (1.0 - SUSTAIN_LEVEL) * (t / DECAY_SECS);
                if level <= SUSTAIN_LEVEL {
                    self.env_phase = EnvPhase::Sustain;
                    SUSTAIN_LEVEL
                } else {
                    level
                }
            }
            EnvPhase::Sustain => {
                // Auto-release if the terminal sends no key-up events
                let sustain_elapsed = self.time - ATTACK_SECS - DECAY_SECS;
                if sustain_elapsed >= MAX_SUSTAIN_SECS {
                    self.release_from = SUSTAIN_LEVEL;
                    self.release_at   = self.time;
                    self.env_phase    = EnvPhase::Release;
                }
                SUSTAIN_LEVEL
            }
            EnvPhase::Release => {
                let t = self.time - self.release_at;
                let level = self.release_from * (1.0 - (t / RELEASE_SECS));
                if level <= 0.001 {
                    self.env_phase = EnvPhase::Off;
                    0.0
                } else {
                    level
                }
            }
            EnvPhase::Off => return 0.0,
        };

        // Additive synthesis: fundamental + 3 harmonics
        let waveform: f32 = HARMONICS
            .iter()
            .map(|(n, amp)| amp * (TAU * self.freq * n * self.time).sin())
            .sum();

        self.time += self.dt;
        waveform * self.env_level * self.velocity
    }
}

# Audio Synthesis Design

## Why Not a Pure Sine Wave?

A pure sine wave at the correct frequency is technically accurate but sounds like a test signal from a signal generator — it has no character. Real piano strings vibrate at a fundamental frequency plus a series of **overtones** (harmonics). It's the blend and decay of those overtones that gives a piano its sound.

## Phase 1: Additive Synthesis

We synthesize a piano-like tone by summing harmonics and shaping the volume over time with an ADSR envelope.

### Waveform Formula

```
sample(t) = A(t) × Σₙ (hₙ × sin(2π × n × f₀ × t + φₙ))

Where:
  t   = time in seconds
  f₀  = fundamental frequency (Hz) of the note
  n   = harmonic number (1, 2, 3, 4)
  hₙ  = amplitude of nth harmonic
  φₙ  = phase offset (0 for our purposes)
  A(t) = ADSR envelope value at time t
```

### Harmonic Series

| Harmonic | Frequency  | Amplitude | Why |
|----------|-----------|-----------|-----|
| 1st (fundamental) | f₀      | 1.000 | The note you hear |
| 2nd overtone       | 2 × f₀  | 0.500 | Octave above — richness |
| 3rd overtone       | 3 × f₀  | 0.250 | Fifth above — warmth |
| 4th overtone       | 4 × f₀  | 0.125 | Two octaves above — brightness |

The amplitudes follow a `1/n` pattern, which is characteristic of triangle/sawtooth wave hybrids and sounds warm and resonant.

### ADSR Envelope

```
Amplitude
  1.0 │    ╱╲
      │   ╱  ╲___________
  0.7 │  ╱               ╲
      │ ╱    ↑    ↑       ╲
      │╱   Decay Sus.    Release
      ├─────────────────────────► time
       Attack
```

| Stage | Duration | Behavior |
|-------|----------|----------|
| **Attack** | 8 ms | Linear ramp 0.0 → 1.0. Fast — gives the "thud" of a piano hammer. |
| **Decay** | 100 ms | Linear ramp 1.0 → sustain level. The initial impact fades. |
| **Sustain** | key-held | Constant at 0.7. While the key is held. |
| **Release** | 250 ms | Linear ramp sustain → 0.0. After key is released. |

### Per-Note Detuning

Pure mathematical frequencies sound too "digital." Real piano strings are slightly detuned from equal temperament, especially in high and low registers (this is called "stretched tuning"). We approximate this with a fixed ±2 cents deviation seeded from the MIDI note number, making it deterministic (same on every run) but unique per note.

```
detune_factor(midi) = 1.0 + (hash(midi) % 4 - 2) * 0.01 / 12.0
```

### Polyphonic Mixing

The `cpal` audio callback runs on a dedicated thread at the hardware sample rate (44100 Hz). It maintains a **voice pool** of up to 16 simultaneous notes. Each voice tracks:
- The note's frequency
- Current ADSR phase and time position
- Whether the key is still held

The callback sums all active voice samples and normalizes to prevent clipping:

```rust
let mixed = voices.iter_mut()
    .map(|v| v.next_sample())
    .sum::<f32>();
let output = (mixed / voices.len().max(1) as f32).clamp(-1.0, 1.0);
```

This is the **only** hot path. It never allocates and never crosses the trait boundary. `AudioPort::play_note` communicates with the voice pool via a lock-free ring buffer (`crossbeam::ArrayQueue`).

---

## Phase 2: SFZ Sample Playback

### Why SFZ

SFZ is an open, text-based format for describing sample instruments. A `.sfz` file maps note ranges and velocity ranges to WAV sample files. It's supported by virtually every professional audio application.

### Salamander Grand Piano

- **Author**: Alexander Holm
- **License**: Creative Commons Attribution 3.0
- **Coverage**: 88 notes, sampled every 3 semitones, 16 velocity layers, 2 round-robins
- **Quality**: 24-bit / 44.1 kHz
- **Size**: ~1.3 GB (full), ~180 MB (16-bit / 44.1 kHz reduced)

This is what MuseScore, Ardour, and many Linux audio setups use as their default piano. It is the best freely available piano sample library.

### Integration Plan

1. `acoustic-audio` gains a `SfzSamplerBackend` struct
2. The SFZ file is parsed to a lookup table: `(note, velocity) → &[f32]` sample slice
3. Samples are decoded at startup (or lazily on first note request)
4. On `play_note`, the matching sample is read into the voice pool with the same ADSR release phase as the synthesizer (for consistent feel)
5. Activated via `--samples /path/to/Salamander.sfz` CLI flag or `~/.config/acoustic-shell/config.toml`

The `AudioPort` interface is **identical**. The engine and UI know nothing about which backend is active.

### Runtime Selection

```
acoustic-shell                          → synthesized (always works)
acoustic-shell --samples ~/piano.sfz    → SFZ sampler
```

The config file can set a default:
```toml
# ~/.config/acoustic-shell/config.toml
[audio]
backend = "sfz"
sfz_path = "~/Salamander/Salamander.sfz"
```

---

## Reference: MIDI Note → Frequency

```
f(n) = 440.0 × 2^((n − 69) / 12)

n=60 (C4, Middle C) → 261.63 Hz
n=69 (A4)           → 440.00 Hz
n=21 (A0, lowest)   → 27.50 Hz
n=108 (C8, highest) → 4186.01 Hz
```

The full formula is implemented in `acoustic-core/src/theory.rs`.

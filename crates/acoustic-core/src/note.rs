use std::fmt;

/// The 12 distinct pitches in Western music. Sharps only — C# = Db, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Pitch {
    C  = 0,
    Cs = 1,
    D  = 2,
    Ds = 3,
    E  = 4,
    F  = 5,
    Fs = 6,
    G  = 7,
    Gs = 8,
    A  = 9,
    As = 10,
    B  = 11,
}

impl Pitch {
    pub fn semitone_index(self) -> u8 {
        self as u8
    }

    pub fn name(self) -> &'static str {
        match self {
            Pitch::C  => "C",
            Pitch::Cs => "C#",
            Pitch::D  => "D",
            Pitch::Ds => "D#",
            Pitch::E  => "E",
            Pitch::F  => "F",
            Pitch::Fs => "F#",
            Pitch::G  => "G",
            Pitch::Gs => "G#",
            Pitch::A  => "A",
            Pitch::As => "A#",
            Pitch::B  => "B",
        }
    }

    /// True for C D E F G A B (the white keys on a piano).
    pub fn is_natural(self) -> bool {
        matches!(self, Pitch::C | Pitch::D | Pitch::E | Pitch::F
                     | Pitch::G | Pitch::A | Pitch::B)
    }

    pub fn from_index(idx: u8) -> Self {
        match idx % 12 {
            0  => Pitch::C,
            1  => Pitch::Cs,
            2  => Pitch::D,
            3  => Pitch::Ds,
            4  => Pitch::E,
            5  => Pitch::F,
            6  => Pitch::Fs,
            7  => Pitch::G,
            8  => Pitch::Gs,
            9  => Pitch::A,
            10 => Pitch::As,
            _  => Pitch::B,
        }
    }

    /// All 12 pitches in chromatic ascending order.
    pub fn all() -> &'static [Pitch; 12] {
        &[
            Pitch::C, Pitch::Cs, Pitch::D, Pitch::Ds,
            Pitch::E, Pitch::F,  Pitch::Fs, Pitch::G,
            Pitch::Gs, Pitch::A, Pitch::As, Pitch::B,
        ]
    }
}

impl fmt::Display for Pitch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// An octave number. Standard piano: A0 (octave 0) to C8 (octave 8).
/// Middle C is C4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Octave(pub i8);

impl Octave {
    pub const MIN: Octave = Octave(0);
    pub const MAX: Octave = Octave(8);

    pub fn clamped(self) -> Self {
        Octave(self.0.max(0).min(8))
    }
}

impl From<i8> for Octave {
    fn from(v: i8) -> Self {
        Octave(v)
    }
}

impl fmt::Display for Octave {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A specific piano key: a pitch in an octave.
///
/// Middle C = Note { pitch: Pitch::C, octave: Octave(4) }
/// MIDI number 60, frequency 261.63 Hz.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Note {
    pub pitch: Pitch,
    pub octave: Octave,
}

impl Note {
    pub fn new(pitch: Pitch, octave: impl Into<Octave>) -> Self {
        Note { pitch, octave: octave.into() }
    }

    /// MIDI note number (0–127). Middle C (C4) = 60. A4 = 69.
    ///
    /// Formula: (octave + 1) * 12 + pitch_semitone
    pub fn midi_number(self) -> u8 {
        let n = (self.octave.0 as i16 + 1) * 12 + self.pitch.semitone_index() as i16;
        n.clamp(0, 127) as u8
    }

    /// Frequency in Hz. Uses equal temperament with A4 = 440 Hz.
    ///
    /// f = 440 × 2^((midi − 69) / 12)
    pub fn frequency_hz(self) -> f32 {
        440.0 * 2f32.powf((self.midi_number() as f32 - 69.0) / 12.0)
    }

    pub fn name(self) -> String {
        format!("{}{}", self.pitch.name(), self.octave.0)
    }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.pitch, self.octave)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn middle_c_midi_60() {
        let c4 = Note::new(Pitch::C, 4i8);
        assert_eq!(c4.midi_number(), 60);
    }

    #[test]
    fn a4_is_440hz() {
        let a4 = Note::new(Pitch::A, 4i8);
        assert!((a4.frequency_hz() - 440.0).abs() < 0.01);
    }

    #[test]
    fn a4_midi_69() {
        let a4 = Note::new(Pitch::A, 4i8);
        assert_eq!(a4.midi_number(), 69);
    }

    #[test]
    fn octave_relationship() {
        // C5 should be exactly double the frequency of C4
        let c4 = Note::new(Pitch::C, 4i8);
        let c5 = Note::new(Pitch::C, 5i8);
        assert!((c5.frequency_hz() / c4.frequency_hz() - 2.0).abs() < 0.001);
    }
}

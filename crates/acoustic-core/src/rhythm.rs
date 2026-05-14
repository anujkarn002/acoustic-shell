/// Tempo in beats per minute.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bpm(pub f32);

impl Default for Bpm {
    fn default() -> Self { Bpm(120.0) }
}

impl Bpm {
    pub fn beat_secs(self) -> f32 {
        60.0 / self.0
    }
}

/// Note duration expressed as a fraction of a whole note (4 beats in 4/4).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Duration {
    Whole,
    Half,
    Quarter,
    Eighth,
    Sixteenth,
    DottedHalf,
    DottedQuarter,
    DottedEighth,
}

impl Duration {
    /// Length in beats (quarter note = 1.0).
    pub fn beats(self) -> f32 {
        match self {
            Duration::Whole         => 4.0,
            Duration::Half          => 2.0,
            Duration::Quarter       => 1.0,
            Duration::Eighth        => 0.5,
            Duration::Sixteenth     => 0.25,
            Duration::DottedHalf    => 3.0,
            Duration::DottedQuarter => 1.5,
            Duration::DottedEighth  => 0.75,
        }
    }

    pub fn secs(self, bpm: Bpm) -> f32 {
        self.beats() * bpm.beat_secs()
    }
}

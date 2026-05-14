use crate::note::{Note, Pitch};
use crate::theory::note_from_midi;

/// Given a slice of simultaneously-sounding notes, return a chord name like
/// "C Major" or "A# m7", or None if fewer than 3 distinct pitch classes.
pub fn detect_chord(notes: &[Note]) -> Option<String> {
    let mut pcs: Vec<u8> = notes.iter().map(|n| n.pitch.semitone_index()).collect();
    pcs.sort_unstable();
    pcs.dedup();
    if pcs.len() < 3 {
        return None;
    }

    const QUALITIES: &[(ChordQuality, &str)] = &[
        (ChordQuality::Major,           ""),
        (ChordQuality::Minor,           "m"),
        (ChordQuality::Diminished,      "dim"),
        (ChordQuality::Augmented,       "aug"),
        (ChordQuality::DominantSeventh, "7"),
        (ChordQuality::MajorSeventh,    "maj7"),
        (ChordQuality::MinorSeventh,    "m7"),
    ];

    for &root_pc in &pcs {
        let intervals: Vec<u8> = pcs.iter()
            .map(|&pc| (pc + 12 - root_pc) % 12)
            .collect();

        for &(quality, suffix) in QUALITIES {
            let target = quality.intervals();
            if target.len() == intervals.len()
                && target.iter().all(|t| intervals.contains(t))
            {
                let root_name = Pitch::from_index(root_pc).name();
                let sep = if suffix.is_empty() { " " } else { "" };
                return Some(format!("{}{}{}", root_name, sep, suffix));
            }
        }
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChordQuality {
    Major,
    Minor,
    Diminished,
    Augmented,
    DominantSeventh,
    MajorSeventh,
    MinorSeventh,
}

impl ChordQuality {
    /// Semitone intervals from the root.
    pub fn intervals(self) -> &'static [u8] {
        match self {
            ChordQuality::Major          => &[0, 4, 7],
            ChordQuality::Minor          => &[0, 3, 7],
            ChordQuality::Diminished     => &[0, 3, 6],
            ChordQuality::Augmented      => &[0, 4, 8],
            ChordQuality::DominantSeventh => &[0, 4, 7, 10],
            ChordQuality::MajorSeventh   => &[0, 4, 7, 11],
            ChordQuality::MinorSeventh   => &[0, 3, 7, 10],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Chord {
    pub root: Note,
    pub quality: ChordQuality,
}

impl Chord {
    pub fn new(root: Note, quality: ChordQuality) -> Self {
        Chord { root, quality }
    }

    pub fn notes(self) -> Vec<Note> {
        let root_midi = self.root.midi_number();
        self.quality.intervals().iter().map(|&interval| {
            note_from_midi((root_midi + interval).min(127))
        }).collect()
    }
}

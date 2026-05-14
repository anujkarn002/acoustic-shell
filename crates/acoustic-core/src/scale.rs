use crate::note::{Note, Pitch};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleKind {
    Major,
    NaturalMinor,
    PentatonicMajor,
    PentatonicMinor,
    Blues,
    Chromatic,
}

impl ScaleKind {
    /// Semitone intervals from the root, ascending.
    pub fn intervals(self) -> &'static [u8] {
        match self {
            ScaleKind::Major          => &[0, 2, 4, 5, 7, 9, 11],
            ScaleKind::NaturalMinor   => &[0, 2, 3, 5, 7, 8, 10],
            ScaleKind::PentatonicMajor => &[0, 2, 4, 7, 9],
            ScaleKind::PentatonicMinor => &[0, 3, 5, 7, 10],
            ScaleKind::Blues          => &[0, 3, 5, 6, 7, 10],
            ScaleKind::Chromatic      => &[0,1,2,3,4,5,6,7,8,9,10,11],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Scale {
    pub root: Pitch,
    pub kind: ScaleKind,
}

impl Scale {
    pub fn new(root: Pitch, kind: ScaleKind) -> Self {
        Scale { root, kind }
    }

    /// Notes in this scale starting from `octave`.
    pub fn notes(self, octave: i8) -> Vec<Note> {
        let root_idx = self.root.semitone_index() as u16 + (octave as u16 + 1) * 12;
        self.kind.intervals().iter().map(|&interval| {
            let midi = (root_idx + interval as u16).min(127) as u8;
            crate::theory::note_from_midi(midi)
        }).collect()
    }
}

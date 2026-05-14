use crate::{note::Note, rhythm::{Bpm, Duration}};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Beginner,
    Intermediate,
    Advanced,
}

/// A single note event in a song: which note, how long, how hard.
#[derive(Debug, Clone)]
pub struct NoteEvent {
    pub note: Note,
    pub duration: Duration,
    pub velocity: u8, // 0–127
}

/// A complete song definition.
#[derive(Debug, Clone)]
pub struct Song {
    pub id: String,
    pub title: String,
    pub artist: Option<String>,
    pub difficulty: Difficulty,
    pub bpm: Bpm,
    pub events: Vec<NoteEvent>,
}

impl Song {
    pub fn total_beats(&self) -> f32 {
        self.events.iter().map(|e| e.duration.beats()).sum()
    }
}

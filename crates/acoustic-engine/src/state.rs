use std::collections::BTreeSet;
use std::sync::Arc;
use acoustic_core::{note::Note, rhythm::Bpm, song::Song};

#[derive(Debug, Clone)]
pub struct AppState {
    pub mode: AppMode,
    pub base_octave: i8,
    pub volume: f32,
    pub bpm: Bpm,
    pub running: bool,
    pub show_help: bool,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            mode: AppMode::FreePlay {
                active_notes: BTreeSet::new(),
                held_notes: BTreeSet::new(),
                sustain: false,
            },
            base_octave: 4,
            volume: 0.7,
            bpm: Bpm::default(),
            running: true,
            show_help: false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum AppMode {
    FreePlay {
        /// All currently sounding notes (physically held + sustained).
        active_notes: BTreeSet<Note>,
        /// Only the physically depressed keys (subset of active_notes).
        held_notes: BTreeSet<Note>,
        sustain: bool,
    },
    GuidedPlay {
        song: Arc<Song>,
        cursor: usize,
        hits: u32,
        misses: u32,
    },
}

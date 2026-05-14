use acoustic_core::note::Note;

/// What the UI needs to render. Built from AppState by the engine.
/// The UI crate never imports acoustic-engine — it only sees this view model.
#[derive(Debug, Clone)]
pub struct UiState {
    pub mode: UiMode,
    pub base_octave: i8,
    pub volume: f32,
    pub status_msg: Option<String>,
    pub show_help: bool,
}

#[derive(Debug, Clone)]
pub enum UiMode {
    FreePlay {
        active_notes: Vec<Note>,
        chord_name: Option<String>,
    },
    GuidedPlay {
        song_title: String,
        current_note: Note,
        key_hint: char,
        upcoming: Vec<Note>,
        progress: (usize, usize), // (done, total)
        hits: u32,
        misses: u32,
    },
    MainMenu {
        items: Vec<String>,
        selected: usize,
    },
}

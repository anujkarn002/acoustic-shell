use acoustic_core::note::Note;

/// All user/system intents. The engine translates raw InputEvents into these.
/// The reducer is the only place that acts on them.
#[derive(Debug, Clone)]
pub enum AppCommand {
    // Note events
    NoteOn(Note),
    NoteOff(Note),
    // Sustain pedal
    SustainOn,
    SustainOff,
    // Global controls
    OctaveUp,
    OctaveDown,
    VolumeUp,
    VolumeDown,
    // UI
    ToggleHelp,
    // Navigation
    Quit,
}

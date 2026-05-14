use acoustic_core::note::Note;

/// Side-effect produced by the engine and dispatched to the AudioPort.
#[derive(Debug, Clone)]
pub enum AudioEffect {
    PlayNote { note: Note, velocity: u8 },
    StopNote(Note),
    SetVolume(f32),
}

/// All events the engine can receive from any input backend.
#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
    /// A character key was pressed down.
    KeyDown(char),
    /// A character key was released (only on terminals with keyboard enhancement).
    KeyUp(char),
    /// Space key pressed — sustain pedal down.
    SustainOn,
    /// Space key released — sustain pedal up.
    SustainOff,
    /// Terminal was resized.
    Resize(u16, u16),
    /// The user requested quit (Ctrl+C, Escape, etc.).
    Quit,
}

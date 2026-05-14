use crate::event::InputEvent;

/// Input boundary — anything that can produce input events.
/// Implemented by CrosstermInput (keyboard) and MiDirInput (MIDI, future).
pub trait InputPort {
    /// Block until the next event arrives.
    fn next_event(&mut self) -> InputEvent;
}

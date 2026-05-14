use crate::state::UiState;

/// Output boundary — anything that can render the application state.
pub trait UiPort {
    fn render(&mut self, state: &UiState) -> anyhow::Result<()>;
}

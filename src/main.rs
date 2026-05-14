use anyhow::Result;
use acoustic_audio::CpalAudioBackend;
use acoustic_input::CrosstermInput;
use acoustic_ui::RatatuiRenderer;

fn main() -> Result<()> {
    // Initialization order matters for Drop:
    // input is declared first → drops last (disables raw mode)
    // ui is declared second  → drops first (leaves alternate screen)
    // This ensures the terminal is restored cleanly on exit.
    let input = CrosstermInput::new()?;
    let ui    = RatatuiRenderer::new()?;
    let audio = CpalAudioBackend::new()?;

    acoustic_engine::run(audio, input, ui)
}

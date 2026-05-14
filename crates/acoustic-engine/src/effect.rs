use acoustic_audio::AudioEffect;

/// Side effects produced by reduce(). Dispatched by the engine AFTER state update.
/// Keeping effects as data means reduce() stays pure and fully testable.
#[derive(Debug, Clone)]
pub enum Effect {
    Audio(AudioEffect),
}

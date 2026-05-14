use acoustic_core::note::Note;

/// Output boundary — anything that can play piano notes.
/// Implemented by CpalAudioBackend (Phase 1) and SfzSamplerBackend (Phase 4).
pub trait AudioPort {
    fn play_note(&mut self, note: Note, velocity: u8);
    fn stop_note(&mut self, note: Note);
    fn set_volume(&mut self, volume: f32);
}

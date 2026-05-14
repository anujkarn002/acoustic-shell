use acoustic_core::song::Song;
use crate::model::SongMeta;

pub trait SongRepository {
    fn list(&self) -> Vec<SongMeta>;
    fn load(&self, id: &str) -> anyhow::Result<Song>;
}

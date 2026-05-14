use acoustic_core::song::Song;
use crate::{
    model::{song_from_file, SongFile, SongMeta},
    port::SongRepository,
};

static TWINKLE: &str = include_str!("../../../assets/songs/twinkle.toml");

/// Song repository backed by files embedded at compile time.
pub struct EmbeddedSongs;

impl EmbeddedSongs {
    pub fn new() -> Self {
        EmbeddedSongs
    }

    fn sources() -> [(&'static str, &'static str); 1] {
        [("twinkle", TWINKLE)]
    }
}

impl SongRepository for EmbeddedSongs {
    fn list(&self) -> Vec<SongMeta> {
        Self::sources()
            .iter()
            .filter_map(|(_, src)| {
                let sf: SongFile = toml::from_str(src).ok()?;
                Some(SongMeta {
                    id: sf.meta.id.clone(),
                    title: sf.meta.title.clone(),
                    difficulty: match sf.meta.difficulty.as_str() {
                        "intermediate" => acoustic_core::song::Difficulty::Intermediate,
                        "advanced"     => acoustic_core::song::Difficulty::Advanced,
                        _              => acoustic_core::song::Difficulty::Beginner,
                    },
                    bpm: sf.meta.bpm,
                })
            })
            .collect()
    }

    fn load(&self, id: &str) -> anyhow::Result<Song> {
        let src = Self::sources()
            .iter()
            .find(|(sid, _)| *sid == id)
            .map(|(_, src)| *src)
            .ok_or_else(|| anyhow::anyhow!("song not found: {id}"))?;
        let sf: SongFile = toml::from_str(src)?;
        song_from_file(sf)
    }
}

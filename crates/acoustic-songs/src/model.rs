use serde::Deserialize;
use acoustic_core::{
    note::{Note, Octave, Pitch},
    rhythm::{Bpm, Duration},
    song::{Difficulty, NoteEvent, Song},
};

/// Lightweight listing info shown in the song browser.
#[derive(Debug, Clone)]
pub struct SongMeta {
    pub id: String,
    pub title: String,
    pub difficulty: Difficulty,
    pub bpm: f32,
}

// ─── TOML wire types ────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct SongFile {
    pub meta: MetaDef,
    pub notes: Vec<NoteDef>,
}

#[derive(Deserialize)]
pub struct MetaDef {
    pub id: String,
    pub title: String,
    pub artist: Option<String>,
    pub difficulty: String,
    pub bpm: f32,
}

#[derive(Deserialize)]
pub struct NoteDef {
    pub pitch: Option<String>, // e.g. "C4", "F#3"
    pub rest: Option<bool>,
    pub duration: String,
}

// ─── Conversion ─────────────────────────────────────────────────────────────

pub fn song_from_file(sf: SongFile) -> anyhow::Result<Song> {
    let bpm = Bpm(sf.meta.bpm);
    let difficulty = match sf.meta.difficulty.as_str() {
        "beginner"     => Difficulty::Beginner,
        "intermediate" => Difficulty::Intermediate,
        _              => Difficulty::Advanced,
    };

    let mut events = Vec::with_capacity(sf.notes.len());
    for nd in sf.notes {
        if nd.rest.unwrap_or(false) {
            continue; // rests not yet tracked in Song; skip
        }
        let pitch_str = nd.pitch.ok_or_else(|| anyhow::anyhow!("note missing pitch"))?;
        let note  = parse_note(&pitch_str)?;
        let dur   = parse_duration(&nd.duration)?;
        events.push(NoteEvent { note, duration: dur, velocity: 90 });
    }

    Ok(Song {
        id: sf.meta.id,
        title: sf.meta.title,
        artist: sf.meta.artist,
        difficulty,
        bpm,
        events,
    })
}

fn parse_note(s: &str) -> anyhow::Result<Note> {
    let (pitch_part, octave_part) = s.split_at(
        if s.len() > 2 && s.as_bytes()[1] == b'#' { 2 } else { 1 },
    );
    let pitch = match pitch_part {
        "C"  => Pitch::C,  "C#" => Pitch::Cs,
        "D"  => Pitch::D,  "D#" => Pitch::Ds,
        "E"  => Pitch::E,
        "F"  => Pitch::F,  "F#" => Pitch::Fs,
        "G"  => Pitch::G,  "G#" => Pitch::Gs,
        "A"  => Pitch::A,  "A#" => Pitch::As,
        "B"  => Pitch::B,
        other => anyhow::bail!("unknown pitch: {other}"),
    };
    let octave: i8 = octave_part.parse()
        .map_err(|_| anyhow::anyhow!("bad octave in '{s}'"))?;
    Ok(Note::new(pitch, Octave(octave)))
}

fn parse_duration(s: &str) -> anyhow::Result<Duration> {
    match s {
        "whole"          => Ok(Duration::Whole),
        "half"           => Ok(Duration::Half),
        "quarter"        => Ok(Duration::Quarter),
        "eighth"         => Ok(Duration::Eighth),
        "sixteenth"      => Ok(Duration::Sixteenth),
        "dotted-half"    => Ok(Duration::DottedHalf),
        "dotted-quarter" => Ok(Duration::DottedQuarter),
        "dotted-eighth"  => Ok(Duration::DottedEighth),
        other => anyhow::bail!("unknown duration: {other}"),
    }
}

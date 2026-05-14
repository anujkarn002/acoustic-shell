# ADR-005: TOML Song Library Format

**Date**: 2026-05-14
**Status**: Accepted

## Context

The guided play mode needs a library of songs represented as sequences of notes with timing. This format needs to be:
- Human-editable (people should be able to write new songs without a special tool)
- Version-control friendly (diff-able, not binary)
- Extensible (add metadata, multiple tracks, lyrics, etc. over time)
- Parseable to the `Song` domain type in `acoustic-core`

## Decision

Use **TOML** as the song definition format.

```toml
[meta]
id          = "twinkle"
title       = "Twinkle Twinkle Little Star"
artist      = "Traditional"
difficulty  = "beginner"    # beginner | intermediate | advanced
bpm         = 100
time_sig    = [4, 4]

[[notes]]
pitch    = "C4"
duration = "quarter"

[[notes]]
pitch    = "C4"
duration = "quarter"

[[notes]]
pitch    = "G4"
duration = "quarter"

# Rests are supported
[[notes]]
rest     = true
duration = "quarter"
```

Duration values: `"whole"`, `"half"`, `"quarter"`, `"eighth"`, `"sixteenth"`, `"dotted-quarter"`, etc.

Songs are embedded in the binary using `include_str!()` / `rust-embed` for the bundled library, and can also be loaded from the filesystem at runtime for user-created songs.

## Alternatives Considered

**Standard MIDI files (.mid)**
- Universal format, tons of songs available
- Binary, not human-editable
- Designed for multi-track orchestration, complex to parse minimally
- Could be added as an import feature later (separate from the primary format)

**JSON**
- Universal, well-tooled
- Verbose for this use case; no comments; TOML is cleaner for config-like data

**YAML**
- Human-readable
- Significant whitespace rules cause subtle bugs; inconsistent multiline string handling
- Not as idiomatic in the Rust ecosystem

**Custom binary format**
- Fastest to parse
- Not human-editable, not diffable, requires dedicated tooling to author songs

**LilyPond / MusicXML**
- Professional music notation formats
- Far too complex for our needs; overkill for a beginner-facing song library

## Consequences

- Contributors can add new songs by writing a TOML file with no code changes
- The TOML → `Song` conversion is a clean boundary tested by `acoustic-songs` unit tests
- MIDI import can be added later as an optional feature (`--import-midi`) that outputs TOML
- Song files are stored in `assets/songs/` and committed to the repository
- Future: a web-based song editor could output this format

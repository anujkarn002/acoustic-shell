# acoustic-shell

Cross-platform terminal music application. Play piano through your keyboard, guided learning mode, beautiful ASCII TUI.

## Architecture

Hexagonal Architecture + Elm state machine + Cargo workspace. See `docs/ARCHITECTURE.md`.

```
main.rs (composition root)
  └── acoustic-engine    (state machine, reduce(), event loop)
        ├── acoustic-core    (domain: Note, Scale, Chord, Song — ZERO external deps)
        ├── acoustic-audio   (AudioPort trait + cpal + ADSR synthesis)
        ├── acoustic-input   (InputPort trait + crossterm)
        ├── acoustic-ui      (UiPort trait + ratatui widgets)
        └── acoustic-songs   (SongRepository trait + TOML loader)
```

## Stack

| What | Crate | Phase |
|------|-------|-------|
| Audio I/O | `cpal` 0.15 | Phase 0 ✅ |
| Synthesis | additive ADSR (custom) | Phase 0 ✅ |
| TUI | `ratatui` 0.29 + `crossterm` 0.28 | Phase 0 ✅ |
| SFZ samples | Salamander Grand Piano | Phase 4 |
| MIDI input | `midir` 0.11 | Phase 5 |
| CLI args | `clap` | Phase 0 ✅ |
| Config/songs | `toml` + `serde` | Phase 0 ✅ |

## Phase Status

| Phase | Goal | Status |
|-------|------|--------|
| 0 | Workspace scaffold + full PoC | ✅ Done (2026-05-14) |
| 1 | Free play polish + proper piano widget | 🔜 Next |
| 2 | TUI polish + full song library | Planned |
| 3 | Guided play mode | Planned |
| 4 | SFZ sampled piano (Salamander Grand Piano) | Planned |
| 5 | MIDI device input | Planned |
| 6 | Tauri GUI / WASM web | Planned |

## WSL2 Audio Setup (required one-time)

cpal uses ALSA; need the PulseAudio bridge plugin:

```bash
sudo pacman -S alsa-plugins

cat >> ~/.asoundrc << 'EOF'
pcm.!default {
    type pulse
    fallback "sysdefault"
}
ctl.!default {
    type pulse
    fallback "sysdefault"
}
EOF
```

- `PULSE_SERVER=unix:/mnt/wslg/PulseServer` is auto-set by WSLg.
- See `docs/adr/006-wsl-audio.md`.

## Dev Commands

```bash
cargo run                           # dev run
cargo build --release               # single binary
cargo test -p acoustic-core         # test a specific crate
cargo test --workspace              # all tests (15 passing)
cross build --release --target x86_64-pc-windows-gnu  # cross-compile
```

## Key Rules

- `acoustic-core` has ZERO external dependencies. Never add any.
- State changes ONLY through `reduce(AppState, AppCommand) → (AppState, Vec<Effect>)`.
- `main.rs` is the only file that knows about concrete adapter types.
- No `unwrap()` outside tests.
- Song files are TOML in `assets/songs/`.
- **Git commits**: never add Claude/AI co-authorship lines. Plain commits only.

## Start of Next Session

1. Fix audio if not done: `sudo pacman -S alsa-plugins` + `~/.asoundrc` (see above)
2. `cargo run` — smoke test, confirm sound + key-release events
3. Upgrade piano widget: proper black/white key ASCII layout (2 octaves)
4. Add sustain pedal (`Space`), active-note flash, chord detection display
5. See `docs/ROADMAP.md` Phase 1 for full task list

## Docs

```
docs/ARCHITECTURE.md   System overview, crate diagram, data flow
docs/SYNTHESIS.md      ADSR synthesis, harmonics, SFZ plan
docs/MUSIC_THEORY.md   Notes, scales, chords, MIDI reference
docs/DEVELOPMENT.md    How to add modes, songs, backends
docs/ROADMAP.md        7-phase delivery plan (Phase 0 done)
docs/adr/              Architecture Decision Records (001–006)
```

## Skill

Type `/acoustic` for architecture guidance, music theory, TUI design, and roadmap tracking.

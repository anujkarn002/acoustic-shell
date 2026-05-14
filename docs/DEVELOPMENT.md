# Development Guide

## Prerequisites

```bash
# Rust toolchain (already confirmed installed)
rustc --version   # 1.94+

# Audio on Arch Linux WSL2 (WSLg ships PulseAudio automatically)
# If ALSA lib is missing:
sudo pacman -S alsa-lib alsa-plugins pulseaudio-alsa

# For release builds targeting other platforms:
cargo install cross
```

## Running

```bash
# Development
cargo run

# Specific mode
cargo run -- --mode free
cargo run -- --mode guided --song twinkle

# With SFZ samples (Phase 4+)
cargo run -- --samples ~/Salamander/Salamander.sfz

# Release binary
cargo build --release
./target/release/acoustic-shell
```

## Testing

```bash
# All tests
cargo test

# Specific crate
cargo test -p acoustic-core
cargo test -p acoustic-engine

# With output (useful for audio integration tests)
cargo test -p acoustic-audio -- --nocapture
```

## Project Structure Quick Reference

```
src/main.rs              Composition root — wires adapters, runs engine
crates/acoustic-core/    Domain model — Note, Scale, Chord, Song, keymap
crates/acoustic-audio/   Audio: synth, ADSR voices, cpal backend
crates/acoustic-input/   Input: crossterm keyboard → InputEvent
crates/acoustic-ui/      UI: ratatui widgets, piano/guided/status views
crates/acoustic-songs/   Song library: TOML format, repository trait
crates/acoustic-engine/  App: state machine, reduce(), event loop
assets/songs/            Bundled TOML song files
docs/                    Architecture, theory, synthesis, ADRs
```

---

## Adding a New AppMode

1. Add variant to `AppMode` enum in `acoustic-engine/src/state.rs`
2. Add command variants to `AppCommand` in `acoustic-engine/src/command.rs`
3. Create `crates/acoustic-engine/src/modes/<name>.rs`
4. Add match arms to `reduce()` in `acoustic-engine/src/reducer.rs`
5. Add UI rendering in `acoustic-ui/src/widgets/<name>.rs`
6. Wire the new widget in `RatatuiRenderer`

The engine and audio crates need no changes.

---

## Adding a New Song

1. Create `assets/songs/<id>.toml`:

```toml
[meta]
id         = "my-song"
title      = "My Song"
difficulty = "beginner"
bpm        = 120
time_sig   = [4, 4]

[[notes]]
pitch    = "C4"
duration = "quarter"

[[notes]]
pitch    = "E4"
duration = "quarter"
```

2. Register it in `acoustic-songs/src/embedded.rs` via `include_str!()`.

That's it. No code changes needed in any other crate.

---

## Adding a New Audio Backend

1. Create `crates/acoustic-audio/src/<name>_backend.rs`
2. Implement `AudioPort` trait:

```rust
pub struct MyBackend { ... }

impl AudioPort for MyBackend {
    fn play_note(&mut self, note: Note, velocity: u8) { ... }
    fn stop_note(&mut self, note: Note) { ... }
    fn set_volume(&mut self, volume: f32) { ... }
}
```

3. Wire it in `src/main.rs` via a CLI flag or config value.

No changes needed in `acoustic-engine`, `acoustic-ui`, or `acoustic-core`.

---

## Adding a New Input Backend

1. Create `crates/acoustic-input/src/<name>_input.rs`
2. Implement `InputPort` trait:

```rust
pub struct MyInput { ... }

impl InputPort for MyInput {
    fn next_event(&mut self) -> InputEvent { ... }
}
```

3. Wire it in `src/main.rs`.

---

## Code Conventions

- No `unwrap()` or `expect()` outside tests — use `?` and `anyhow::Result`
- No `Arc<Mutex<T>>` in the engine — state flows through `reduce()`, not shared pointers
- No comments explaining what code does — names do that; comments explain WHY (non-obvious constraints, workarounds)
- `acoustic-core` must stay dependency-free — never add an external crate to its `Cargo.toml`
- All public types in `acoustic-core` implement `Debug`, `Clone`, `PartialEq`
- Song TOML files use `kebab-case` IDs

---

## Cross-Platform Builds

```bash
# Linux x86_64 (native)
cargo build --release

# Windows (from Linux)
cargo install cross
cross build --release --target x86_64-pc-windows-gnu

# macOS (from Linux — requires osxcross toolchain)
cross build --release --target x86_64-apple-darwin
```

---

## WSL2 Audio Notes

Audio works via WSLg PulseAudio — no configuration needed. The `PULSE_SERVER` env var is auto-set by WSLg to `unix:/mnt/wslg/PulseServer`.

If audio stops working after a WSL restart:
```bash
# Check if PulseAudio server is up
pactl info

# If not, WSLg may need to be restarted — exit WSL and run from PowerShell:
wsl --shutdown
wsl
```

See `docs/adr/006-wsl-audio.md` for full details.

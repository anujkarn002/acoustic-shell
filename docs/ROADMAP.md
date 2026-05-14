# acoustic-shell — Roadmap

Each phase is independently releasable. The architecture (Hexagonal + Elm state machine) ensures that later phases add new code without modifying existing crates.

---

## Phase 0 — Foundation ✅ COMPLETE (2026-05-14)

**Goal**: Prove the audio pipeline works and establish workspace structure.

- [x] Confirm WSLg PulseAudio audio output on WSL2
- [x] Architecture design, ADRs, full documentation suite
- [x] Cargo workspace with 6 crates (`acoustic-core`, `-audio`, `-input`, `-ui`, `-songs`, `-engine`)
- [x] `acoustic-core`: `Note`, `Pitch`, `Octave`, frequency formula, keymap, scale, chord, theory
- [x] `acoustic-audio`: ADSR voice synthesizer (4 harmonics + per-note detuning), polyphonic cpal backend, lock-free command channel
- [x] `acoustic-input`: crossterm raw mode + keyboard enhancement (key-release events)
- [x] `acoustic-engine`: Elm-style `reduce()`, full event loop, AppState → UiState projection
- [x] `acoustic-ui`: ratatui renderer, piano widget, status bar, free play + guided play + menu views
- [x] `acoustic-songs`: embedded TOML song library, Twinkle Twinkle Little Star
- [x] `src/main.rs`: 10-line composition root wiring all adapters
- [x] 15 unit tests passing, zero warnings

**Success criteria met**: Binary compiles and is ready to play. Press A → C4, Z/X → octave, -/+ → volume, ESC → quit.

---

## Phase 1 — Free Play Polish *(next)*

**Goal**: First run smoke test + visual and UX polish for free play mode.

### First session tasks (start here)
- [ ] **Smoke test**: `cargo run` — verify sound plays on WSL2, check latency, check key-release
- [ ] **Piano widget**: upgrade from 12-cell row to proper black/white key layout (2 octave view)
- [ ] **Active note flash**: brief color animation when a note is struck
- [ ] **Sustain pedal**: hold `Space` to sustain all notes (delay release until Space released)
- [ ] **Help overlay**: `?` key toggles a key-binding reference panel
- [ ] **Frequency display**: show Hz of the last-played note in the status bar
- [ ] **Terminal resize**: gracefully redraw on resize without crash

### Stretch goals for Phase 1
- [ ] Chord display: detect and name chords when 3+ notes are held (e.g., "C Major")
- [ ] Scroll/pan across more than one octave without changing base octave
- [ ] Config file: `~/.config/acoustic-shell/config.toml` for default volume, octave, keymap

**Success**: A musician sitting down cold can understand the interface and play freely within 30 seconds.

---

## Phase 2 — TUI Polish & Song Library

**Goal**: Beautiful terminal visuals and a working song library.

- [ ] ASCII logo / splash screen with animation on startup
- [ ] Full 88-key piano display (scrollable or condensed)
- [ ] Note ripple/flash animation on keypress
- [ ] VU meter / waveform visualization widget
- [ ] `acoustic-songs`: TOML format, `SongRepository` trait, `TomlSongRepository`
- [ ] Bundle beginner song pack:
  - Twinkle Twinkle Little Star
  - Happy Birthday
  - Ode to Joy
  - Mary Had a Little Lamb
  - Für Elise (simplified)
  - Smoke on the Water (riff)
- [ ] Song browser UI (navigate list, preview metadata)

**Success**: Beautiful TUI that rivals professional terminal apps (like `btop` aesthetics).

---

## Phase 3 — Guided Play Mode

**Goal**: Interactive learning mode — notes appear sequentially for the user to hit.

- [ ] `acoustic-engine`: GuidedPlay mode, scoring (hits/misses/streak)
- [ ] `acoustic-ui`: GuidedPlay widget
  - Current note prompt: large display + key to press
  - Upcoming notes preview (next 8)
  - Progress bar (notes completed / total)
  - Score display
  - Visual flash on correct hit, shake on wrong hit
- [ ] Timing-tolerant mode (hit the note, timing optional) for beginners
- [ ] Timed mode (must hit note within the beat window) for intermediate
- [ ] End-of-song score screen with stats
- [ ] Practice mode: loop a section, slow down BPM

**Success**: A user who has never played piano can learn "Twinkle Twinkle" in one session.

---

## Phase 4 — Natural Sound (SFZ Samples)

**Goal**: Real piano quality via Salamander Grand Piano sample pack.

- [ ] `acoustic-audio`: `SfzSamplerBackend: impl AudioPort`
  - SFZ parser (subset: opcode key, lovel, hivel, sample, pitch_keycenter)
  - WAV sample loading (using `hound`)
  - Velocity-sensitive sample selection
  - Round-robin variation to avoid "machine gun" effect on repeated notes
- [ ] `--samples <path>` CLI flag
- [ ] Config file: `~/.config/acoustic-shell/config.toml`
- [ ] Document sample download instructions in README

**Success**: Running with `--samples ~/Salamander.sfz` sounds like a real piano.

---

## Phase 5 — MIDI Device Input

**Goal**: Connect a MIDI keyboard or controller as input.

- [ ] `acoustic-input`: `MiDirInput: impl InputPort` using the `midir` crate
- [ ] MIDI port enumeration and selection (`--midi-port <name>`)
- [ ] Velocity-sensitive note playback (MIDI velocity → amplitude)
- [ ] Pitch bend wheel support
- [ ] Sustain pedal (CC64) support
- [ ] MIDI clock sync (optional, for BPM sync with DAW)

**Success**: Plugging in a USB MIDI keyboard works with `--midi-port <device>`.

---

## Phase 6 — Extended Modes

**Goal**: Richer learning and creation features.

- [ ] **Scale practice**: display a scale, guide user through it, show which notes belong
- [ ] **Chord builder**: show chord voicings, highlight on keyboard
- [ ] **Ear training**: play a note/chord, user identifies it
- [ ] **Recording**: record a free-play session to a TOML song file
- [ ] **Metronome**: audible click track with configurable BPM
- [ ] **Transposition**: shift all notes up/down N semitones

---

## Phase 7 — GUI & Web Bindings

**Goal**: Non-terminal interfaces that reuse all existing business logic.

- [ ] **Tauri desktop app**: `acoustic-ui` gets a `TauriRenderer: impl UiPort`; same engine, same songs, same audio
- [ ] **WebAssembly**: `acoustic-audio` gets a `WebAudioBackend: impl AudioPort` using the Web Audio API; compile the engine to WASM; build a browser-based version

These are pure **new adapter crates**. `acoustic-core` and `acoustic-engine` compile as-is to WASM.

---

## Non-Goals (explicitly out of scope)

- **DAW / sequencer**: This is a learning + performance instrument, not a full digital audio workstation
- **Multi-track composition**: One instrument at a time is the focus
- **Audio recording to file**: Out of scope initially (may revisit in Phase 6)
- **Network multiplayer**: Interesting future idea, not planned

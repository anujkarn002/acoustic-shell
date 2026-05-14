# acoustic-shell — Architecture

## Overview

acoustic-shell is a cross-platform terminal music application. It starts as a CLI piano and is designed from the ground up to support multiple frontends (TUI, GUI, web), multiple input devices (keyboard, MIDI), and multiple audio backends (synthesis, samples). The architecture makes adding any of these zero-impact on everything else.

---

## Core Principles

### 1. Hexagonal Architecture (Ports & Adapters)

The application domain — notes, state, rules, song logic — sits at the center and knows nothing about terminals, audio hardware, or files. External systems (cpal, ratatui, crossterm, MIDI devices) are behind trait interfaces called **ports**. Concrete implementations are **adapters**.

```
                    ┌─────────────────────────────────┐
                    │           Domain Core            │
                    │  (acoustic-core, acoustic-engine) │
                    │  No I/O. No external crate deps.  │
                    └────────────┬────────────┬────────┘
                                 │            │
               ┌─────────────────┘            └──────────────────┐
               ▼                                                  ▼
   ┌───────────────────────┐                     ┌───────────────────────────┐
   │    Input Ports        │                     │      Output Ports         │
   │  trait InputPort      │                     │  trait AudioPort          │
   │  trait SongRepository │                     │  trait UiPort             │
   └───────────┬───────────┘                     └─────────────┬─────────────┘
               │                                               │
   ┌───────────┴───────────┐                     ┌─────────────┴─────────────┐
   │      Adapters         │                     │         Adapters           │
   │  CrosstermInput       │                     │  CpalAudioBackend          │
   │  MiDirInput (future)  │                     │  SfzSamplerBackend (ph2)  │
   │  TomlSongRepository   │                     │  RatatuiRenderer           │
   └───────────────────────┘                     │  TauriRenderer (future)    │
                                                 └────────────────────────────┘
```

### 2. Elm-Style State Machine

Application state is a plain value — no shared mutable state, no callbacks, no interior mutability in the core. The **only** way state changes is through a pure function:

```
reduce(AppState, AppCommand) → (AppState, Vec<Effect>)
```

`AppCommand` is intent (what the user wants). `Effect` is a side effect to be dispatched after the state update (play a note, stop a note, quit). The core logic is deterministic and testable without any I/O.

### 3. Cargo Workspace

Six focused crates with hard compile-time boundaries. No crate can accidentally depend on the wrong layer — the dependency graph is enforced by Cargo. The binary (`src/main.rs`) is the only place that wires concrete adapters together.

---

## Workspace Layout

```
acoustic-shell/
│
├── Cargo.toml                    ← workspace root
├── CLAUDE.md                     ← project quick-reference
│
├── src/
│   └── main.rs                   ← binary entry point (~30 lines)
│
├── crates/
│   ├── acoustic-core/            ← pure domain, zero external deps
│   ├── acoustic-audio/           ← AudioPort trait + cpal/synth adapter
│   ├── acoustic-input/           ← InputPort trait + crossterm adapter
│   ├── acoustic-ui/              ← UiPort trait + ratatui adapter + widgets
│   ├── acoustic-songs/           ← SongRepository trait + TOML loader
│   └── acoustic-engine/          ← App state machine + orchestration
│
├── assets/
│   └── songs/                    ← bundled TOML song definitions
│
└── docs/
    ├── ARCHITECTURE.md           ← this file
    ├── DEVELOPMENT.md            ← contributor guide
    ├── MUSIC_THEORY.md           ← music theory reference
    ├── SYNTHESIS.md              ← audio synthesis design
    ├── ROADMAP.md                ← phased delivery plan
    └── adr/                      ← Architecture Decision Records
```

---

## Dependency Graph

Direction of arrows = "depends on". No cycles are permitted.

```
main (binary)
  └── acoustic-engine
        ├── acoustic-core         (no external deps)
        ├── acoustic-audio   ──►  acoustic-core
        ├── acoustic-input   ──►  acoustic-core
        ├── acoustic-ui      ──►  acoustic-core
        └── acoustic-songs   ──►  acoustic-core
```

`acoustic-engine` depends on the **trait types** defined in each adapter crate, not the concrete implementations. `main.rs` is the composition root — it instantiates the concrete adapters and passes them to the engine.

---

## Crate Reference

### `acoustic-core`

The domain model. Contains every concept that is true regardless of how the app is run.

| Module | Responsibility |
|--------|---------------|
| `note` | `Note { pitch, octave }`, MIDI number, Hz frequency |
| `interval` | Semitone intervals, `Interval` enum |
| `scale` | `Scale { root, kind }` → ordered note sequences |
| `chord` | `Chord { root, quality }` → note sets |
| `rhythm` | `Duration`, `TimeSignature`, `Bpm` |
| `song` | `Song`, `Track`, `NoteEvent { note, duration, velocity }` |
| `keymap` | QWERTY layout → Note translation (layout-configurable) |
| `theory` | `frequency_hz(note)`, `note_name(midi)`, `midi_number(note)` |

**Zero external dependencies.** Every function is unit-testable.

### `acoustic-audio`

Owns the audio abstraction and all synthesis logic.

| Module | Responsibility |
|--------|---------------|
| `port` | `trait AudioPort { play_note, stop_note, set_volume }` |
| `effect` | `AudioEffect` enum (what the engine dispatches) |
| `synth` | Additive synthesis: 4 harmonics + ADSR envelope per voice |
| `voice` | `Voice` — a single active note with envelope state |
| `mixer` | Polyphonic voice mixing for the cpal callback |
| `cpal_backend` | `CpalAudioBackend: impl AudioPort` |

See `docs/SYNTHESIS.md` for the full synthesis design.

### `acoustic-input`

Owns input abstraction and keyboard-to-note translation.

| Module | Responsibility |
|--------|---------------|
| `port` | `trait InputPort`, `InputEvent` enum |
| `event` | `InputEvent { KeyDown(Key), KeyUp(Key), Resize(u16, u16), Quit }` |
| `terminal` | `CrosstermInput: impl InputPort` — raw mode, event polling |

### `acoustic-ui`

Owns rendering abstraction and all visual components.

| Module | Responsibility |
|--------|---------------|
| `port` | `trait UiPort { render(&AppState) → Result<()> }` |
| `widgets/piano` | 88-key piano layout, active note highlighting, key labels |
| `widgets/guided` | Next-note prompt, upcoming notes preview, progress bar |
| `widgets/status` | Mode, BPM, octave, score |
| `widgets/splash` | Main menu with ASCII logo |
| `terminal` | `RatatuiRenderer: impl UiPort` |

### `acoustic-songs`

Manages the song library.

| Module | Responsibility |
|--------|---------------|
| `port` | `trait SongRepository { list() → Vec<SongMeta>, load(id) → Result<Song> }` |
| `model` | `SongDef`, `NoteDef` — serde-friendly wire types |
| `toml` | `TomlSongRepository` — reads from embedded + filesystem assets |
| `convert` | `SongDef → Song` (wire type → domain type) |

### `acoustic-engine`

The application. Owns the event loop and state machine. No direct I/O.

| Module | Responsibility |
|--------|---------------|
| `app` | `App` struct — holds state + trait object refs, runs the loop |
| `state` | `AppState`, `AppMode` enum |
| `command` | `AppCommand` enum — all possible user/system intents |
| `effect` | `Effect` enum — side effects as data |
| `reducer` | `fn reduce(AppState, AppCommand) → (AppState, Vec<Effect>)` |
| `modes/free_play` | FreePlay mode: keyboard → note on/off |
| `modes/guided` | GuidedPlay: note prompt, hit detection, scoring |

---

## Key Data Types

### `Note`
```rust
pub struct Note {
    pub pitch: Pitch,   // C, Cs (C#), D, Ds, E, F, Fs, G, Gs, A, As, B
    pub octave: Octave, // i8 newtype, valid range 0..=8
}

impl Note {
    pub fn midi_number(&self) -> u8
    pub fn frequency_hz(&self) -> f32   // 440.0 * 2^((midi - 69) / 12)
}
```

### `AppState`
```rust
pub struct AppState {
    pub mode: AppMode,
    pub base_octave: Octave,
    pub volume: f32,       // 0.0..=1.0
    pub bpm: Bpm,
}

pub enum AppMode {
    MainMenu { selected: usize },
    FreePlay { active_notes: BTreeSet<Note> },
    GuidedPlay {
        song: Arc<Song>,
        cursor: usize,          // index into song.events
        hits: u32,
        misses: u32,
    },
}
```

### `AppCommand`
```rust
pub enum AppCommand {
    // Navigation
    SelectMode(AppMode),
    Quit,
    // Free play
    NoteOn(Note),
    NoteOff(Note),
    // Guided play
    Attempt(Note),
    // Global
    OctaveUp,
    OctaveDown,
    VolumeUp,
    VolumeDown,
}
```

### `Effect`
```rust
pub enum Effect {
    Audio(AudioEffect),
    Ui(UiEffect),
}

pub enum AudioEffect {
    PlayNote { note: Note, velocity: u8, duration_ms: Option<u32> },
    StopNote(Note),
    SetVolume(f32),
}

pub enum UiEffect {
    Flash(Note),    // visual feedback on correct guided hit
    Shake,          // visual feedback on wrong hit
}
```

---

## Event Loop

```rust
// acoustic-engine/src/app.rs (simplified)
pub fn run(mut state: AppState, input: &dyn InputPort,
           audio: &mut dyn AudioPort, ui: &mut dyn UiPort) {
    loop {
        ui.render(&state);

        let input_event = input.next_event();
        if let Some(cmd) = translate(input_event, &state) {
            let (new_state, effects) = reduce(state, cmd);
            state = new_state;
            for effect in effects {
                dispatch(effect, audio, ui);
            }
        }
    }
}
```

The `reduce` and `translate` functions are pure — no I/O, no side effects. All integration testing of application logic goes through them directly.

---

## Testing Strategy

| Crate | Approach | Key assertions |
|-------|----------|---------------|
| `acoustic-core` | Pure unit tests | Note math, scale/chord generation, key mapping |
| `acoustic-engine` | Unit tests with mock traits | `reduce()` produces correct state + effects for every (mode, command) pair |
| `acoustic-audio` | Integration: render to buffer | ADSR envelope shape, correct peak frequency, no clipping |
| `acoustic-ui` | Snapshot tests | Widget render output matches expected ASCII buffer |
| `acoustic-songs` | Unit tests | TOML parse produces correct domain `Song` |

Mock adapters live in `acoustic-engine/src/testing.rs`:
```rust
pub struct MockAudio { pub effects: Vec<AudioEffect> }
impl AudioPort for MockAudio { ... }

pub struct MockUi { pub render_count: usize }
impl UiPort for MockUi { ... }
```

---

## Extension Points

| Future feature | Where | Impact on existing code |
|----------------|-------|------------------------|
| MIDI controller input | New adapter in `acoustic-input` | None |
| Tauri GUI | New adapter in `acoustic-ui` | None |
| WebAssembly / web | New audio + UI adapters | None |
| SFZ sampled piano | New backend in `acoustic-audio` | None |
| New instrument (guitar, drums) | New `InstrumentKind` in core + new keymap | Additive |
| Recording / playback | New `AppMode::Recording` in engine | Additive |
| Multiplayer | New `NetworkPort` trait | Additive |

---

## Binary Entry Point

`src/main.rs` is the composition root. It is the **only** place that knows about all concrete adapter types.

```rust
fn main() -> anyhow::Result<()> {
    let args = cli::parse();
    let audio = CpalAudioBackend::new()?;
    let input = CrosstermInput::new()?;
    let ui    = RatatuiRenderer::new()?;
    let songs = TomlSongRepository::embedded();
    acoustic_engine::run(audio, input, ui, songs, args.into())
}
```

Everything else is trait objects. `main.rs` never contains logic.

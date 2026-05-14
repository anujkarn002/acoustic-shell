# ADR-002: Hexagonal Architecture (Ports & Adapters)

**Date**: 2026-05-14
**Status**: Accepted

## Context

acoustic-shell will eventually support:
- Multiple input sources: QWERTY keyboard, MIDI controller, touch screen
- Multiple audio backends: synthesized tones, SFZ samples, WebAudio API
- Multiple frontends: terminal TUI, Tauri desktop GUI, browser WASM

Without a deliberate architecture, adding any of these would require touching the core application logic. The risk is tight coupling between the domain (what notes to play, how scoring works) and the infrastructure (how to render pixels, how to produce audio bytes).

## Decision

Apply **Hexagonal Architecture**: the domain sits at the center and all external systems are behind **port interfaces** (Rust traits). Concrete implementations are **adapters** that live in their own crates.

```
Ports:
  trait AudioPort       — play_note, stop_note, set_volume
  trait InputPort       — next_event() → InputEvent
  trait UiPort          — render(&AppState)
  trait SongRepository  — list(), load(id)

Adapters (impl the ports):
  CpalAudioBackend      → impl AudioPort
  SfzSamplerBackend     → impl AudioPort   (Phase 2)
  CrosstermInput        → impl InputPort
  MiDirInput            → impl InputPort   (Phase 5)
  RatatuiRenderer       → impl UiPort
  TauriRenderer         → impl UiPort      (Phase 6)
  TomlSongRepository    → impl SongRepository
```

The engine (`acoustic-engine`) depends only on the port traits, never on concrete adapters. `main.rs` is the only place that knows about concrete types and wires them together.

## Alternatives Considered

**Direct coupling** — engine calls `cpal` directly.
- Fast to start
- Adding a GUI requires rewriting large sections of the engine
- Untestable without hardware

**Observer / callback pattern** — register callbacks for audio events.
- More familiar to GUI developers
- Harder to reason about execution order
- Callbacks are difficult to test cleanly

**Actor model** — each subsystem is a concurrent actor (like Tokio tasks).
- Good for distributed systems
- Overhead for a single-process desktop app
- Makes deterministic testing harder

## Consequences

- The entire engine + domain are testable with `MockAudio` and `MockUi` — no hardware needed
- Swapping the audio backend for Phase 2 (SFZ samples) requires zero changes to `acoustic-engine`
- Port traits define the contract; adapters must fulfill it
- Slight indirection cost via `dyn Trait` — acceptable for a UI-bound application where audio callback performance matters only in the hot path (which bypasses the trait anyway)
- The hot audio path (the cpal callback) does not go through `AudioPort` — it talks directly to the mixer buffer. `AudioPort::play_note` just enqueues a command that the mixer reads. So there is zero overhead on the actual sample generation path.

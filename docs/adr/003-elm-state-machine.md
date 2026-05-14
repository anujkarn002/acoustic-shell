# ADR-003: Elm-Style State Machine for Application Logic

**Date**: 2026-05-14
**Status**: Accepted

## Context

acoustic-shell has real-time state: active notes, guided play position, score, current mode. This state must update in response to keyboard events, timing events, and MIDI events. Without a deliberate approach, application logic scatters across event handlers, the UI layer, and the audio callback — making it impossible to test and hard to reason about.

## Decision

Apply the **Elm Architecture** (also called Redux, or the Message-Update pattern) for all application state:

```rust
fn reduce(state: AppState, command: AppCommand) -> (AppState, Vec<Effect>)
```

- `AppState` is a plain immutable value (or `Clone`-able struct)
- `AppCommand` expresses user intent (what should happen, not how)
- `Vec<Effect>` lists side effects to dispatch after the state update
- `reduce` is a **pure function** — no I/O, no global state, fully deterministic

The engine calls `reduce`, takes the new state and effects, then dispatches effects to the appropriate ports.

```
InputEvent
    │
    ▼
translate_to_command(&state, event) → Option<AppCommand>
    │
    ▼
reduce(state, command) → (new_state, Vec<Effect>)      ← pure
    │
    ├─► dispatch AudioEffect → AudioPort
    └─► dispatch UiEffect    → UiPort
```

## Alternatives Considered

**Shared mutable state with callbacks**
- Simplest to write initially
- State can be modified from any layer — hard to trace bugs
- Requires `Arc<Mutex<>>` for multi-thread access, which is error-prone

**Actor model (message passing between concurrent actors)**
- Good isolation
- Complex for a single-user desktop app
- Testing requires spinning up actors

**ECS (Entity-Component-System, like Bevy)**
- Powerful for game-like apps
- Heavy dependency
- Overkill for a focused instrument app

## Consequences

- `reduce()` is the most-tested function in the codebase — call it with any `(state, command)` pair and assert on output
- No test requires audio hardware, a terminal, or file I/O
- Adding a new mode = adding a new `AppMode` variant + a `reduce` match arm
- State history / undo is trivially implementable (keep a `Vec<AppState>` stack)
- Effects as data makes logging, replay, and debugging straightforward — print `Vec<Effect>` and you see exactly what the app decided to do

## Note on Performance

`AppState` is `Clone`-d on every state transition. For a music app this is fine: state transitions happen at human input speeds (1-100/sec), not audio sample speeds (44100/sec). The audio callback operates entirely outside this path on a separate thread.

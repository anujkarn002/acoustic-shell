# ADR-001: Cargo Workspace with Multiple Crates

**Date**: 2026-05-14
**Status**: Accepted

## Context

acoustic-shell needs to be:
- CLI-first, but adaptable to GUI (Tauri) and web (WASM) without rewriting the core
- Testable without audio hardware or a terminal
- Cross-platform with simple binary distribution

The first structural decision is whether to organize the codebase as a **single crate with modules** or a **Cargo workspace with multiple crates**.

## Decision

Use a **Cargo workspace** with six crates from day one:

```
acoustic-core / acoustic-audio / acoustic-input /
acoustic-ui / acoustic-songs / acoustic-engine
```

Plus `src/main.rs` as the binary entry point.

## Alternatives Considered

**Option A — Single crate, internal modules**
- Pro: simpler `Cargo.toml`, faster to start
- Con: no compile-time enforcement of layer boundaries; a future GUI crate would have to depend on the whole thing; the "domain has no I/O deps" invariant is a convention not a guarantee

**Option B — Two crates: core + app**
- Pro: cleaner than one, still simple
- Con: not fine-grained enough to swap individual adapters (audio, UI, input) independently

**Option C — Workspace (chosen)**
- Pro: each crate's `Cargo.toml` enforces exactly which dependencies that layer is allowed; can publish crates individually; `acoustic-core` having zero external deps is a hard guarantee; adding Tauri means adding one new crate, not modifying existing ones
- Con: more `Cargo.toml` files upfront; slightly more `use` statements

## Consequences

- `acoustic-core` **cannot** accidentally import `cpal` or `ratatui` — Cargo won't allow it
- Integration tests live inside each crate under `tests/`
- The composition root (`main.rs`) is the only file that sees all concrete types
- Adding a new frontend or backend = adding a new crate, zero changes to existing crates
- Binary is still a single compiled executable; workspace doesn't affect the end artifact

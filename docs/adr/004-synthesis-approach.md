# ADR-004: Hybrid Audio Synthesis Approach

**Date**: 2026-05-14
**Status**: Accepted

## Context

The goal is a natural, high-quality piano sound. There are three broad strategies:

1. **Pure synthesis** — generate tones mathematically, no sample files
2. **Sample playback** — record a real piano, play back the recordings
3. **Hybrid** — start with synthesis, upgrade to samples as an optional backend

The user specifically asked for "best in class and natural" sound, so quality matters. But we also need the app to work immediately without downloading large asset files.

## Decision

**Hybrid approach in two phases:**

### Phase 1 — Additive Synthesis with ADSR

Generate piano-like tones in software using additive synthesis. No sample files required.

**Waveform:**
```
f(t) = A(t) × Σ hₙ × sin(2π × n × f₀ × t)

Harmonics (n, amplitude hₙ):
  1st (fundamental): 1.0
  2nd harmonic:      0.5
  3rd harmonic:      0.25
  4th harmonic:      0.125

f₀ = note frequency in Hz = 440 × 2^((midi - 69) / 12)
```

**ADSR envelope A(t):**
```
  Attack:  8ms    (fast attack for percussive piano feel)
  Decay:   100ms  (fall from peak to sustain level)
  Sustain: 0.7    (70% amplitude while key held)
  Release: 250ms  (fade after key released)
```

**Per-note detuning:** Each note gets ±1-2 cents of random detuning to avoid the "perfect digital sine" feel. Calculated once at startup and seeded from the MIDI note number (deterministic).

This produces an "electric piano" or "digital piano" quality — clearly musical and pleasant, not a test tone.

### Phase 2 — SFZ Sample Playback (Salamander Grand Piano)

**Salamander Grand Piano** is the gold standard of free piano samples:
- License: Creative Commons Attribution 3.0
- Coverage: 88 notes × 16 velocity layers × 2 round-robins
- Format: SFZ (open format, plain text mapping + WAV samples)
- Size: ~1.3 GB full, ~180 MB at reduced sample rate

Integration plan:
- `acoustic-audio` gains a `SfzSamplerBackend: impl AudioPort`
- Samples are loaded lazily per note (not all 88 × 16 at startup)
- At runtime: `--samples /path/to/Salamander` flag activates sampler mode
- Synthesized mode remains available as default and for offline/resource-constrained use

The `AudioPort` trait is identical for both backends — the engine doesn't know or care which is active.

## Alternatives Considered

**Pure synthesis only**
- Simple, zero assets
- Doesn't achieve "natural" quality
- Acceptable for MVP but not the final vision

**Samples from day one**
- Best quality immediately
- Requires users to download ~180MB before first use
- Blocks initial development on asset pipeline work

**FM synthesis (frequency modulation)**
- More expressive than additive, less CPU than samples
- Requires careful operator tuning per instrument
- Complex to implement well; good for electric pianos/organs, harder for acoustic piano

**Use an existing synthesizer library (e.g., `fundsp`)**
- Powerful DSP toolkit
- Would work, but adds a heavy dependency and reduces our understanding of the audio pipeline
- Building the synthesizer ourselves is educational and gives full control for Phase 2 integration

## Consequences

- Phase 1 works immediately on any machine, zero assets
- The synthesizer code in `acoustic-audio/src/synth.rs` is self-contained and well-documented
- Upgrading to samples requires adding `SfzSamplerBackend`, zero changes to the engine or UI
- Users can choose synthesis vs. samples at launch time via a CLI flag or config
- The audio architecture (voice pool, ADSR state, polyphonic mixer) is shared between both backends

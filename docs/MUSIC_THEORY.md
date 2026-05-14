# Music Theory Reference

This document is a concise music theory reference for contributors and for understanding the domain model in `acoustic-core`. You don't need to know music to contribute — read this once and the code will make sense.

---

## Notes

There are **12 distinct pitches** in Western music, arranged in a repeating pattern called an **octave**:

```
C  C#  D  D#  E  F  F#  G  G#  A  A#  B  │  C  C#  D ...
─────────────────────────────────────────┘  └─────────────
                 one octave                   next octave
```

The `#` symbol means **sharp** — one half-step higher. Some notes have two names (enharmonic equivalents): `C# = Db`, `D# = Eb`, etc. In our codebase we use sharps consistently.

A **half step** (semitone) = moving one position in the 12-note sequence.
A **whole step** (tone) = two half steps.

---

## Octaves and Middle C

The same 12 notes repeat in **octaves**. Each octave is twice the frequency of the one below. We number them 0 through 8.

| Note | MIDI# | Frequency |
|------|-------|-----------|
| A0 (lowest piano note) | 21 | 27.5 Hz |
| C4 **"Middle C"** | 60 | 261.63 Hz |
| A4 **"Concert A" (tuning standard)** | 69 | 440.00 Hz |
| C8 (highest piano note) | 108 | 4186 Hz |

**Frequency formula**: `f = 440 × 2^((midi - 69) / 12)`

A standard piano has **88 keys**: A0 to C8.

---

## MIDI Note Numbers

MIDI is the universal music protocol (1983, still in use everywhere). Each note is a number 0–127:
- 0 = C-1 (sub-bass, below piano range)
- 21 = A0 (lowest piano key)
- 60 = C4 = Middle C
- 69 = A4 = 440 Hz
- 108 = C8 (highest piano key)
- 127 = G9

Notes also have **velocity** (0–127): how hard the key is struck. 0 = silence, 127 = maximum force. A piano plays louder and brighter at higher velocities.

---

## Keyboard Layout (QWERTY → Piano)

```
Keyboard:  [ W ][ E ]   [ T ][ Y ][ U ]   ← black keys (sharps)
           [ A ][ S ][ D ][ F ][ G ][ H ][ J ]   ← white keys

Piano:       C#  D#       F#  G#  A#
             C   D   E    F   G   A   B
```

| Key | Note | MIDI | Hz |
|-----|------|------|----|
| A | C4 | 60 | 261.6 |
| W | C#4 | 61 | 277.2 |
| S | D4 | 62 | 293.7 |
| E | D#4 | 63 | 311.1 |
| D | E4 | 64 | 329.6 |
| F | F4 | 65 | 349.2 |
| T | F#4 | 66 | 370.0 |
| G | G4 | 67 | 392.0 |
| Y | G#4 | 68 | 415.3 |
| H | A4 | 69 | 440.0 |
| U | A#4 | 70 | 466.2 |
| J | B4 | 71 | 493.9 |
| K | C5 | 72 | 523.3 |

`Z` = octave down, `X` = octave up.

---

## Scales

A **scale** is a selection of notes from the 12 that sound good together. Most Western music uses 7-note scales.

### Major Scale

The "happy" scale. Pattern: **W W H W W W H** (whole/half steps)

Example: **C Major** — C D E F G A B

```
C  D  E  F  G  A  B  C
│  │  │  │  │  │  │
W  W  H  W  W  W  H
```

Most pop songs, nursery rhymes, and folk songs use a major scale.

### Natural Minor Scale

The "sad" or "dark" scale. Pattern: **W H W W H W W**

Example: **A Minor** — A B C D E F G

### Pentatonic Scale (5 notes)

Remove the two "tense" notes from a major scale. What remains sounds great over almost any chord — perfect for improvisation.

**C Major Pentatonic**: C D E G A (skip F and B)

Every guitar solo you've ever heard probably uses a pentatonic scale.

### Blues Scale (6 notes)

Pentatonic + one "blue note" (the flattened 5th). The sound of blues, jazz, and rock.

**C Blues**: C Eb F F# G Bb

---

## Chords

A **chord** is 3+ notes played simultaneously.

### Triads (3 notes)

**Major chord**: root + 4 semitones + 3 semitones (R + M3 + m3)
```
C major: C (60) + E (64) + G (67)
         └──4───┘ └──3───┘
```
Sound: bright, happy, stable.

**Minor chord**: root + 3 semitones + 4 semitones
```
C minor: C (60) + Eb (63) + G (67)
         └──3───┘ └───4──┘
```
Sound: dark, melancholic.

**Diminished**: root + 3 + 3 — tense, unstable
**Augmented**: root + 4 + 4 — dreamy, unresolved

### The I-IV-V Progression

The most common chord progression in Western music (literally thousands of pop songs):

In C major: **C major (I) → F major (IV) → G major (V)**

These three chords together contain all 7 notes of the C major scale.

---

## Rhythm

**BPM (beats per minute)**: the tempo. 60 BPM = 1 beat/second.
- Slow ballad: 60–80 BPM
- Pop / rock: 100–140 BPM
- Techno: 130–160 BPM

**Time signature**: `4/4` means 4 beats per measure, each beat is a quarter note. Most Western music is 4/4.

**Note durations** (in 4/4 time):
| Name | Beats | Symbol |
|------|-------|--------|
| Whole | 4 | `━━━━` |
| Half | 2 | `━━` |
| Quarter | 1 | `♩` |
| Eighth | 0.5 | `♪` |
| Sixteenth | 0.25 | `𝅘𝅥𝅯` |

A **dotted** note adds 50% of the base duration (dotted quarter = 1.5 beats).

---

## MIDI Protocol Basics

MIDI (Musical Instrument Digital Interface) is a protocol for sending music events between devices. Every MIDI message has:
- **Status byte**: message type (Note On, Note Off, Control Change, etc.)
- **Data bytes**: usually note number (0–127) and velocity (0–127)

**Note On** (0x90): start playing a note
**Note Off** (0x80): stop playing a note

When we add MIDI controller support (`midir`), these are the messages we'll receive. The `acoustic-input` crate will translate them into the same `InputEvent` type used by keyboard input.

---

## Intervals

The distance between two notes, measured in semitones:

| Semitones | Interval Name | Example (from C) |
|-----------|---------------|-----------------|
| 0 | Unison | C → C |
| 1 | Minor 2nd | C → C# |
| 2 | Major 2nd | C → D |
| 3 | Minor 3rd | C → Eb |
| 4 | Major 3rd | C → E |
| 5 | Perfect 4th | C → F |
| 6 | Tritone | C → F# |
| 7 | Perfect 5th | C → G |
| 8 | Minor 6th | C → Ab |
| 9 | Major 6th | C → A |
| 10 | Minor 7th | C → Bb |
| 11 | Major 7th | C → B |
| 12 | Octave | C → C (higher) |

These are represented as the `Interval` enum in `acoustic-core/src/interval.rs`.

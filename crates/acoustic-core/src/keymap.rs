use crate::note::{Note, Octave, Pitch};

/// Translates a QWERTY character to a piano Note at the given base octave.
///
/// Layout (two rows covering one octave + a few extra):
///
/// ```text
///   W  E     T  Y  U       ← black keys (sharps)
///  A  S  D  F  G  H  J  K  ← white keys C D E F G A B C(+1)
/// ```
///
/// Z = octave down (not a note)
/// X = octave up   (not a note)
pub fn key_to_note(ch: char, base_octave: i8) -> Option<Note> {
    let (pitch, delta): (Pitch, i8) = match ch.to_ascii_lowercase() {
        // White keys — lower row
        'a' => (Pitch::C,  0),
        's' => (Pitch::D,  0),
        'd' => (Pitch::E,  0),
        'f' => (Pitch::F,  0),
        'g' => (Pitch::G,  0),
        'h' => (Pitch::A,  0),
        'j' => (Pitch::B,  0),
        'k' => (Pitch::C,  1),  // C one octave above
        'l' => (Pitch::D,  1),
        ';' => (Pitch::E,  1),
        // Black keys — upper row
        'w' => (Pitch::Cs, 0),
        'e' => (Pitch::Ds, 0),
        't' => (Pitch::Fs, 0),
        'y' => (Pitch::Gs, 0),
        'u' => (Pitch::As, 0),
        'o' => (Pitch::Cs, 1),
        'p' => (Pitch::Ds, 1),
        _ => return None,
    };
    let octave = Octave((base_octave + delta).clamp(0, 8));
    Some(Note::new(pitch, octave))
}

/// Returns the keyboard character hint for a given note at the given base octave.
/// Used by the UI to display "[A]" next to C4, etc.
pub fn note_to_key(note: Note, base_octave: i8) -> Option<char> {
    let delta = note.octave.0 - base_octave;
    match (note.pitch, delta) {
        (Pitch::C,  0) => Some('A'),
        (Pitch::D,  0) => Some('S'),
        (Pitch::E,  0) => Some('D'),
        (Pitch::F,  0) => Some('F'),
        (Pitch::G,  0) => Some('G'),
        (Pitch::A,  0) => Some('H'),
        (Pitch::B,  0) => Some('J'),
        (Pitch::C,  1) => Some('K'),
        (Pitch::D,  1) => Some('L'),
        (Pitch::E,  1) => Some(';'),
        (Pitch::Cs, 0) => Some('W'),
        (Pitch::Ds, 0) => Some('E'),
        (Pitch::Fs, 0) => Some('T'),
        (Pitch::Gs, 0) => Some('Y'),
        (Pitch::As, 0) => Some('U'),
        (Pitch::Cs, 1) => Some('O'),
        (Pitch::Ds, 1) => Some('P'),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_key_gives_c4_at_octave_4() {
        let note = key_to_note('a', 4).unwrap();
        assert_eq!(note.pitch, Pitch::C);
        assert_eq!(note.octave.0, 4);
    }

    #[test]
    fn w_key_gives_c_sharp() {
        let note = key_to_note('w', 4).unwrap();
        assert_eq!(note.pitch, Pitch::Cs);
    }

    #[test]
    fn roundtrip_key_note_key() {
        for ch in "asdfghjwetyu".chars() {
            let note = key_to_note(ch, 4).unwrap();
            let back = note_to_key(note, 4).unwrap();
            assert_eq!(back.to_ascii_lowercase(), ch);
        }
    }
}

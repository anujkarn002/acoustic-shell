use crate::note::{Note, Octave, Pitch};

/// Reconstruct a Note from a raw MIDI note number (0–127).
pub fn note_from_midi(midi: u8) -> Note {
    let semitone = midi % 12;
    let octave   = (midi as i8 / 12) - 1;
    Note::new(Pitch::from_index(semitone), Octave(octave))
}

/// Semitones between two notes (signed: positive = second is higher).
pub fn semitones_between(from: Note, to: Note) -> i16 {
    to.midi_number() as i16 - from.midi_number() as i16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn midi_60_is_c4() {
        let note = note_from_midi(60);
        assert_eq!(note.pitch, Pitch::C);
        assert_eq!(note.octave.0, 4);
    }

    #[test]
    fn semitones_c4_to_a4_is_9() {
        let c4 = Note::new(Pitch::C, 4i8);
        let a4 = Note::new(Pitch::A, 4i8);
        assert_eq!(semitones_between(c4, a4), 9);
    }
}

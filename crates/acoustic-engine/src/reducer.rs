use std::collections::BTreeSet;
use acoustic_audio::AudioEffect;
use crate::{
    command::AppCommand,
    effect::Effect,
    state::{AppMode, AppState},
};

/// Pure state transition. No I/O, no side effects.
/// Returns the new state and a list of effects to dispatch.
pub fn reduce(mut state: AppState, cmd: AppCommand) -> (AppState, Vec<Effect>) {
    match cmd {
        AppCommand::Quit => {
            (AppState { running: false, ..state }, vec![])
        }

        AppCommand::ToggleHelp => {
            (AppState { show_help: !state.show_help, ..state }, vec![])
        }

        AppCommand::OctaveUp => {
            let (stopped, stop_effects) = stop_all_notes(&state);
            let new_octave = (stopped.base_octave + 1).min(7);
            (AppState { base_octave: new_octave, ..stopped }, stop_effects)
        }

        AppCommand::OctaveDown => {
            let (stopped, stop_effects) = stop_all_notes(&state);
            let new_octave = (stopped.base_octave - 1).max(0);
            (AppState { base_octave: new_octave, ..stopped }, stop_effects)
        }

        AppCommand::VolumeUp => {
            let vol = (state.volume + 0.1).min(1.0);
            let fx  = vec![Effect::Audio(AudioEffect::SetVolume(vol))];
            (AppState { volume: vol, ..state }, fx)
        }

        AppCommand::VolumeDown => {
            let vol = (state.volume - 0.1).max(0.0);
            let fx  = vec![Effect::Audio(AudioEffect::SetVolume(vol))];
            (AppState { volume: vol, ..state }, fx)
        }

        AppCommand::SustainOn => {
            if let AppMode::FreePlay { ref mut sustain, .. } = state.mode {
                *sustain = true;
            }
            (state, vec![])
        }

        AppCommand::SustainOff => {
            if let AppMode::FreePlay { ref mut active_notes, ref held_notes, ref mut sustain } = state.mode {
                *sustain = false;
                let to_stop: Vec<_> = active_notes.iter()
                    .filter(|n| !held_notes.contains(*n))
                    .copied()
                    .collect();
                for note in &to_stop {
                    active_notes.remove(note);
                }
                let fx = to_stop.into_iter()
                    .map(|n| Effect::Audio(AudioEffect::StopNote(n)))
                    .collect();
                return (state, fx);
            }
            (state, vec![])
        }

        AppCommand::NoteOn(note) => note_on(state, note),
        AppCommand::NoteOff(note) => note_off(state, note),
    }
}

// ─── mode-specific helpers ───────────────────────────────────────────────────

fn note_on(mut state: AppState, note: acoustic_core::note::Note) -> (AppState, Vec<Effect>) {
    match &mut state.mode {
        AppMode::FreePlay { active_notes, held_notes, .. } => {
            active_notes.insert(note);
            held_notes.insert(note);
            let fx = vec![Effect::Audio(AudioEffect::PlayNote { note, velocity: 90 })];
            (state, fx)
        }
        AppMode::GuidedPlay { song, cursor, hits, misses } => {
            let expected = song.events.get(*cursor).map(|e| e.note);
            let total    = song.events.len();

            if Some(note) == expected {
                *hits  += 1;
                *cursor += 1;
                let fx = vec![Effect::Audio(AudioEffect::PlayNote { note, velocity: 100 })];
                if *cursor >= total {
                    let new_state = AppState {
                        mode: AppMode::FreePlay {
                            active_notes: BTreeSet::new(),
                            held_notes: BTreeSet::new(),
                            sustain: false,
                        },
                        ..state.clone()
                    };
                    (new_state, fx)
                } else {
                    (state, fx)
                }
            } else {
                *misses += 1;
                (state, vec![])
            }
        }
    }
}

fn note_off(mut state: AppState, note: acoustic_core::note::Note) -> (AppState, Vec<Effect>) {
    match &mut state.mode {
        AppMode::FreePlay { active_notes, held_notes, sustain } => {
            held_notes.remove(&note);
            if *sustain {
                // Note remains sounding (sustained); audio continues
                (state, vec![])
            } else if active_notes.remove(&note) {
                (state, vec![Effect::Audio(AudioEffect::StopNote(note))])
            } else {
                (state, vec![])
            }
        }
        AppMode::GuidedPlay { .. } => (state, vec![]),
    }
}

fn stop_all_notes(state: &AppState) -> (AppState, Vec<Effect>) {
    let mut fx = vec![];
    let new_mode = match &state.mode {
        AppMode::FreePlay { active_notes, .. } => {
            for &note in active_notes {
                fx.push(Effect::Audio(AudioEffect::StopNote(note)));
            }
            AppMode::FreePlay {
                active_notes: BTreeSet::new(),
                held_notes: BTreeSet::new(),
                sustain: false,
            }
        }
        other => other.clone(),
    };
    (AppState { mode: new_mode, ..state.clone() }, fx)
}

// ─── tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use acoustic_core::note::{Note, Octave, Pitch};
    use acoustic_audio::AudioEffect;

    fn c4() -> Note { Note::new(Pitch::C, Octave(4)) }

    #[test]
    fn note_on_produces_play_effect() {
        let state = AppState::default();
        let (new_state, fx) = reduce(state, AppCommand::NoteOn(c4()));
        assert!(matches!(fx[0], Effect::Audio(AudioEffect::PlayNote { .. })));
        match &new_state.mode {
            AppMode::FreePlay { active_notes, .. } => assert!(active_notes.contains(&c4())),
            _ => panic!("wrong mode"),
        }
    }

    #[test]
    fn note_off_produces_stop_effect() {
        let state = AppState::default();
        let (state, _) = reduce(state, AppCommand::NoteOn(c4()));
        let (_, fx) = reduce(state, AppCommand::NoteOff(c4()));
        assert!(matches!(fx[0], Effect::Audio(AudioEffect::StopNote(_))));
    }

    #[test]
    fn note_on_replays_after_auto_release() {
        let state = AppState::default();
        let (state, _) = reduce(state, AppCommand::NoteOn(c4()));
        let (_, fx) = reduce(state, AppCommand::NoteOn(c4()));
        assert!(matches!(fx[0], Effect::Audio(AudioEffect::PlayNote { .. })));
    }

    #[test]
    fn octave_up_clears_active_notes() {
        let state = AppState::default();
        let (state, _) = reduce(state, AppCommand::NoteOn(c4()));
        let (new_state, fx) = reduce(state, AppCommand::OctaveUp);
        assert_eq!(new_state.base_octave, 5);
        assert!(fx.iter().any(|f| matches!(f, Effect::Audio(AudioEffect::StopNote(_)))));
    }

    #[test]
    fn volume_clamps_at_1() {
        let state = AppState { volume: 0.95, ..Default::default() };
        let (new_state, _) = reduce(state, AppCommand::VolumeUp);
        assert!(new_state.volume <= 1.0);
    }

    #[test]
    fn quit_sets_running_false() {
        let state = AppState::default();
        let (new_state, _) = reduce(state, AppCommand::Quit);
        assert!(!new_state.running);
    }

    #[test]
    fn sustain_holds_note_after_key_release() {
        let state = AppState::default();
        let (state, _) = reduce(state, AppCommand::SustainOn);
        let (state, _) = reduce(state, AppCommand::NoteOn(c4()));
        let (state, fx) = reduce(state, AppCommand::NoteOff(c4()));
        assert!(fx.is_empty(), "no StopNote while sustain is held");
        match &state.mode {
            AppMode::FreePlay { active_notes, .. } => assert!(active_notes.contains(&c4())),
            _ => panic!("wrong mode"),
        }
    }

    #[test]
    fn sustain_off_releases_sustained_notes() {
        let state = AppState::default();
        let (state, _) = reduce(state, AppCommand::SustainOn);
        let (state, _) = reduce(state, AppCommand::NoteOn(c4()));
        let (state, _) = reduce(state, AppCommand::NoteOff(c4()));
        let (state, fx) = reduce(state, AppCommand::SustainOff);
        assert!(fx.iter().any(|f| matches!(f, Effect::Audio(AudioEffect::StopNote(_)))));
        match &state.mode {
            AppMode::FreePlay { active_notes, .. } => assert!(!active_notes.contains(&c4())),
            _ => panic!("wrong mode"),
        }
    }
}

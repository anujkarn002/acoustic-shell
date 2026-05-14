
use acoustic_audio::{AudioEffect, AudioPort};
use acoustic_input::{InputEvent, InputPort};
use acoustic_ui::{
    state::{UiMode, UiState},
    UiPort,
};
use acoustic_core::{chord::detect_chord, keymap::key_to_note};
use crate::{
    command::AppCommand,
    effect::Effect,
    reducer::reduce,
    state::{AppMode, AppState},
};

/// Run the application. Blocks until the user quits.
///
/// This is the composition root's entry point. It owns the event loop and
/// wires the three ports together. No concrete types — only traits.
pub fn run(
    mut audio: impl AudioPort,
    mut input: impl InputPort,
    mut ui:    impl UiPort,
) -> anyhow::Result<()> {
    let mut state = AppState::default();
    audio.set_volume(state.volume);

    loop {
        ui.render(&to_ui_state(&state))?;

        if !state.running {
            break;
        }

        let event = input.next_event();

        if let Some(cmd) = translate(&state, event) {
            let (new_state, effects) = reduce(state, cmd);
            state = new_state;
            for effect in effects {
                dispatch(effect, &mut audio);
            }
        }
    }

    Ok(())
}

// ─── Input translation ───────────────────────────────────────────────────────

fn translate(state: &AppState, event: InputEvent) -> Option<AppCommand> {
    match event {
        InputEvent::Quit          => Some(AppCommand::Quit),
        InputEvent::KeyDown(ch)   => translate_down(state, ch),
        InputEvent::KeyUp(ch)     => translate_up(state, ch),
        InputEvent::SustainOn     => Some(AppCommand::SustainOn),
        InputEvent::SustainOff    => Some(AppCommand::SustainOff),
        InputEvent::Resize(_, _)  => None,
    }
}

fn translate_down(state: &AppState, ch: char) -> Option<AppCommand> {
    match ch.to_ascii_lowercase() {
        'z'       => Some(AppCommand::OctaveDown),
        'x'       => Some(AppCommand::OctaveUp),
        '-'       => Some(AppCommand::VolumeDown),
        '=' | '+' => Some(AppCommand::VolumeUp),
        '?'       => Some(AppCommand::ToggleHelp),
        _         => key_to_note(ch, state.base_octave).map(AppCommand::NoteOn),
    }
}

fn translate_up(state: &AppState, ch: char) -> Option<AppCommand> {
    key_to_note(ch, state.base_octave).map(AppCommand::NoteOff)
}

// ─── Effect dispatch ─────────────────────────────────────────────────────────

fn dispatch(effect: Effect, audio: &mut impl AudioPort) {
    match effect {
        Effect::Audio(AudioEffect::PlayNote { note, velocity }) => {
            audio.play_note(note, velocity);
        }
        Effect::Audio(AudioEffect::StopNote(note)) => {
            audio.stop_note(note);
        }
        Effect::Audio(AudioEffect::SetVolume(v)) => {
            audio.set_volume(v);
        }
    }
}

// ─── State → view model ──────────────────────────────────────────────────────

fn to_ui_state(state: &AppState) -> UiState {
    let mode = match &state.mode {
        AppMode::FreePlay { active_notes, .. } => {
            let notes: Vec<_> = active_notes.iter().copied().collect();
            let chord_name = detect_chord(&notes);
            UiMode::FreePlay { active_notes: notes, chord_name }
        }
        AppMode::GuidedPlay { song, cursor, hits, misses } => {
            let events   = &song.events;
            let current  = events.get(*cursor).map(|e| e.note)
                .unwrap_or_else(|| acoustic_core::note::Note::new(
                    acoustic_core::note::Pitch::C,
                    acoustic_core::note::Octave(4),
                ));
            let key_hint = acoustic_core::keymap::note_to_key(current, state.base_octave)
                .unwrap_or('?');
            let upcoming = events.iter()
                .skip(*cursor + 1)
                .take(8)
                .map(|e| e.note)
                .collect();
            UiMode::GuidedPlay {
                song_title: song.title.clone(),
                current_note: current,
                key_hint,
                upcoming,
                progress: (*cursor, events.len()),
                hits: *hits,
                misses: *misses,
            }
        }
    };

    UiState {
        mode,
        base_octave: state.base_octave,
        volume: state.volume,
        status_msg: None,
        show_help: state.show_help,
    }
}

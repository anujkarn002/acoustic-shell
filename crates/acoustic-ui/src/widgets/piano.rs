use acoustic_core::note::{Note, Octave, Pitch};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

// ─── Layout constants ────────────────────────────────────────────────────────
//
// 10 white keys (C D E F G A B | C D E), 4 chars each → 40 chars total.
// 7 black keys (C# D# F# G# A# | C# D#), 3 chars each, overlapping white
// key boundaries.
//
// Black-key row char map (0-indexed, 40 chars):
//   [0-1]   C  visible left  (2)
//   [2-4]   C# black         (3)
//   [5]     D  visible       (1)
//   [6-8]   D# black         (3)
//   [9-11]  E  visible right (3)
//   [12-13] F  visible left  (2)
//   [14-16] F# black         (3)
//   [17]    G  visible       (1)
//   [18-20] G# black         (3)
//   [21]    A  visible       (1)
//   [22-24] A# black         (3)
//   [25-27] B  visible right (3)
//   [28-29] C5 visible left  (2)
//   [30-32] C#5 black        (3)
//   [33]    D5 visible       (1)
//   [34-36] D#5 black        (3)
//   [37-39] E5 visible right (3)

const WHITE_KEYS: &[(Pitch, i8, &str)] = &[
    (Pitch::C, 0, "A"), (Pitch::D, 0, "S"), (Pitch::E, 0, "D"),
    (Pitch::F, 0, "F"), (Pitch::G, 0, "G"), (Pitch::A, 0, "H"),
    (Pitch::B, 0, "J"), (Pitch::C, 1, "K"), (Pitch::D, 1, "L"),
    (Pitch::E, 1, ";"),
];

// ─── Rendering ───────────────────────────────────────────────────────────────

pub fn render_piano(active: &[Note], base_octave: i8) -> Vec<Line<'static>> {
    let active_set: std::collections::HashSet<Note> =
        active.iter().copied().collect();

    let is_active = |pitch: Pitch, delta: i8| -> bool {
        active_set.contains(&Note::new(pitch, Octave(base_octave + delta)))
    };

    let white_style = |pitch: Pitch, delta: i8| -> Style {
        if is_active(pitch, delta) {
            Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD)
        } else {
            Style::default().bg(Color::White).fg(Color::DarkGray)
        }
    };

    let black_style = |pitch: Pitch, delta: i8| -> Style {
        if is_active(pitch, delta) {
            Style::default().bg(Color::Yellow).fg(Color::Black).add_modifier(Modifier::BOLD)
        } else {
            Style::default().bg(Color::DarkGray).fg(Color::Gray)
        }
    };

    // ── Row 0: black key overlay ──────────────────────────────────────────────
    let bk_row = vec![
        Span::styled("  ",  white_style(Pitch::C,  0)), // C  left  (2)
        Span::styled(" W ", black_style(Pitch::Cs, 0)), // C#       (3)
        Span::styled(" ",   white_style(Pitch::D,  0)), // D  mid   (1)
        Span::styled(" E ", black_style(Pitch::Ds, 0)), // D#       (3)
        Span::styled("   ", white_style(Pitch::E,  0)), // E  right (3)
        Span::styled("  ",  white_style(Pitch::F,  0)), // F  left  (2)
        Span::styled(" T ", black_style(Pitch::Fs, 0)), // F#       (3)
        Span::styled(" ",   white_style(Pitch::G,  0)), // G  mid   (1)
        Span::styled(" Y ", black_style(Pitch::Gs, 0)), // G#       (3)
        Span::styled(" ",   white_style(Pitch::A,  0)), // A  mid   (1)
        Span::styled(" U ", black_style(Pitch::As, 0)), // A#       (3)
        Span::styled("   ", white_style(Pitch::B,  0)), // B  right (3)
        Span::styled("  ",  white_style(Pitch::C,  1)), // C5 left  (2)
        Span::styled(" O ", black_style(Pitch::Cs, 1)), // C#5      (3)
        Span::styled(" ",   white_style(Pitch::D,  1)), // D5 mid   (1)
        Span::styled(" P ", black_style(Pitch::Ds, 1)), // D#5      (3)
        Span::styled("   ", white_style(Pitch::E,  1)), // E5 right (3)
    ];

    // ── Row 1: white key hints (keyboard letter, 4 chars per key) ─────────────
    let hint_row: Vec<Span> = WHITE_KEYS.iter().map(|&(pitch, delta, hint)| {
        Span::styled(format!(" {:<3}", hint), white_style(pitch, delta))
    }).collect();

    // ── Row 2: white key note names (4 chars per key) ─────────────────────────
    let note_row: Vec<Span> = WHITE_KEYS.iter().map(|&(pitch, delta, _)| {
        let name = format!("{}{}", pitch.name(), base_octave + delta);
        Span::styled(format!("{:<4}", name), white_style(pitch, delta))
    }).collect();

    vec![
        Line::from(bk_row),
        Line::from(hint_row),
        Line::from(note_row),
    ]
}

pub fn piano_widget(active: &[Note], base_octave: i8) -> Paragraph<'static> {
    Paragraph::new(render_piano(active, base_octave))
}

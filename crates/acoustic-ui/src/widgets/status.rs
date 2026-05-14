use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

pub fn status_line(base_octave: i8, volume: f32, mode_name: &str) -> Line<'static> {
    let accent = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
    let dim    = Style::default().fg(Color::DarkGray);

    Line::from(vec![
        Span::styled(" ♪ acoustic-shell ", accent),
        Span::styled("│ ", dim),
        Span::styled(format!("Oct: {base_octave}"), Style::default().fg(Color::Yellow)),
        Span::styled("  │  ", dim),
        Span::styled(format!("Vol: {}%", (volume * 100.0) as u8), Style::default().fg(Color::Green)),
        Span::styled("  │  ", dim),
        Span::styled(mode_name.to_string(), Style::default().fg(Color::White)),
        Span::styled("  │  Z/X: octave  -/+: vol  ESC: quit ", dim),
    ])
}

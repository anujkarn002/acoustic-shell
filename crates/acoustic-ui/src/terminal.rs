use std::io::{self, Stdout};
use crossterm::{execute, terminal::{EnterAlternateScreen, LeaveAlternateScreen}};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame, Terminal,
};
use acoustic_core::note::Note;
use crate::{
    port::UiPort,
    state::{UiMode, UiState},
    widgets::{piano::render_piano, status::status_line},
};

pub struct RatatuiRenderer {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl RatatuiRenderer {
    pub fn new() -> anyhow::Result<Self> {
        execute!(io::stdout(), EnterAlternateScreen)?;
        let backend  = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }
}

impl Drop for RatatuiRenderer {
    fn drop(&mut self) {
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
    }
}

impl UiPort for RatatuiRenderer {
    fn render(&mut self, state: &UiState) -> anyhow::Result<()> {
        self.terminal.draw(|f| draw(f, state))?;
        Ok(())
    }
}

fn draw(f: &mut Frame, state: &UiState) {
    let area = f.area();

    let outer = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));
    f.render_widget(outer, area);

    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    // Vertical layout: status | content | piano (5 rows: border + 3 content + 1 spacer)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // status bar
            Constraint::Min(3),    // content / splash
            Constraint::Length(5), // piano keyboard (1 border + 3 rows + 1 spare)
        ])
        .split(inner);

    // Status bar
    let mode_name = match &state.mode {
        UiMode::FreePlay { .. }   => "free play",
        UiMode::GuidedPlay { .. } => "guided play",
        UiMode::MainMenu { .. }   => "menu",
    };
    f.render_widget(
        Paragraph::new(status_line(state.base_octave, state.volume, mode_name)),
        chunks[0],
    );

    // Content area
    match &state.mode {
        UiMode::FreePlay { active_notes, chord_name } => {
            draw_free_play(f, chunks[1], active_notes, chord_name.as_deref());
        }
        UiMode::GuidedPlay { song_title, current_note, key_hint, upcoming, progress, hits, misses } => {
            draw_guided(f, chunks[1], song_title, *current_note, *key_hint, upcoming, *progress, *hits, *misses);
        }
        UiMode::MainMenu { items, selected } => {
            draw_menu(f, chunks[1], items, *selected);
        }
    }

    // Piano keyboard
    let active = match &state.mode {
        UiMode::FreePlay { active_notes, .. } => active_notes.clone(),
        _ => vec![],
    };

    let piano_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            " keyboard ",
            Style::default().fg(Color::DarkGray),
        ));
    f.render_widget(piano_block, chunks[2]);

    let piano_inner = Rect {
        x: chunks[2].x + 1,
        y: chunks[2].y + 1,
        width: chunks[2].width.saturating_sub(2),
        height: chunks[2].height.saturating_sub(1),
    };
    let piano_lines = render_piano(&active, state.base_octave);
    f.render_widget(Paragraph::new(piano_lines), piano_inner);

    // Help overlay (rendered on top of everything else)
    if state.show_help {
        draw_help(f, area);
    }
}

fn draw_free_play(f: &mut Frame, area: Rect, active: &[Note], chord_name: Option<&str>) {
    let mut lines = vec![Line::from("")];

    if active.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("  Play any key  ", Style::default().fg(Color::DarkGray)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Space = sustain  ?  = help", Style::default().fg(Color::DarkGray)),
        ]));
    } else {
        let names: Vec<String> = active.iter().map(|n| n.name()).collect();
        lines.push(Line::from(vec![
            Span::styled("  Playing: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                names.join("  "),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
        ]));

        if let Some(name) = chord_name {
            lines.push(Line::from(vec![
                Span::styled("  Chord: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    name.to_string(),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
            ]));
        }
    }

    f.render_widget(Paragraph::new(lines), area);
}

fn draw_guided(
    f: &mut Frame, area: Rect,
    song_title: &str, current: Note, hint: char,
    upcoming: &[Note], progress: (usize, usize),
    hits: u32, misses: u32,
) {
    let up_names: Vec<String> = upcoming.iter().take(8).map(|n| n.name()).collect();
    let bar_width = (area.width as usize).saturating_sub(4);
    let filled = if progress.1 > 0 {
        bar_width * progress.0 / progress.1
    } else { 0 };
    let bar = format!("{}{}", "█".repeat(filled), "░".repeat(bar_width - filled));

    let lines = vec![
        Line::from(vec![
            Span::styled(format!("  ♪ {song_title}"), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Coming up: ", Style::default().fg(Color::DarkGray)),
            Span::styled(up_names.join("  "), Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ▶  Hit ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("[ {} ]", current.name()),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("  press  [{hint}]"), Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("  {bar}  {}/{}", progress.0, progress.1),
                Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled(format!("  ✓ {hits}   ✗ {misses}"), Style::default().fg(Color::DarkGray)),
        ]),
    ];
    f.render_widget(Paragraph::new(lines), area);
}

fn draw_menu(f: &mut Frame, area: Rect, items: &[String], selected: usize) {
    let splash = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            r"   __ _  ___ ___  _   _ ___ _   ___ ",
            Style::default().fg(Color::Cyan),
        )]),
        Line::from(vec![Span::styled(
            r"  / _` |/ __/ _ \| | | / __| | / __|",
            Style::default().fg(Color::Cyan),
        )]),
        Line::from(vec![Span::styled(
            r" | (_| | (_| (_) | |_| \__ \ || (__ ",
            Style::default().fg(Color::Cyan),
        )]),
        Line::from(vec![Span::styled(
            r"  \__,_|\___\___/ \__,_|___/_| \___|",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
    ];

    let mut lines = splash;
    for (i, item) in items.iter().enumerate() {
        if i == selected {
            lines.push(Line::from(vec![
                Span::styled(format!("  ▶  {item}"),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled(format!("     {item}"),
                    Style::default().fg(Color::DarkGray)),
            ]));
        }
    }

    f.render_widget(Paragraph::new(lines).alignment(Alignment::Left), area);
}

fn draw_help(f: &mut Frame, area: Rect) {
    let popup = centered_rect(50, 14, area);
    f.render_widget(Clear, popup);

    let block = Block::default()
        .title(Span::styled(" Help ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));
    f.render_widget(block, popup);

    let inner = Rect {
        x: popup.x + 2,
        y: popup.y + 1,
        width: popup.width.saturating_sub(4),
        height: popup.height.saturating_sub(2),
    };

    let col = Style::default().fg(Color::DarkGray);
    let key = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
    let txt = Style::default().fg(Color::White);

    let row = |k: &'static str, desc: &'static str| {
        Line::from(vec![
            Span::styled(format!("{:<20}", k), key),
            Span::styled(desc, txt),
        ])
    };

    let lines = vec![
        Line::from(Span::styled("  Piano keys", col)),
        row("  A S D F G H J K L ;", "white keys (C–E)"),
        row("  W E   T Y U   O P",   "black keys (sharps)"),
        Line::from(""),
        Line::from(Span::styled("  Controls", col)),
        row("  Space",               "sustain pedal"),
        row("  Z / X",               "octave down / up"),
        row("  - / =",               "volume down / up"),
        row("  ?",                   "toggle this help"),
        row("  Esc / Ctrl+C",        "quit"),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}

/// Return a centered Rect of fixed width chars and height rows within `area`.
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect {
        x,
        y,
        width: width.min(area.width),
        height: height.min(area.height),
    }
}

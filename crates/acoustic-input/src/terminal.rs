use std::collections::HashMap;
use std::io;
use std::time::{Duration, Instant};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
            KeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
            PopKeyboardEnhancementFlags},
    execute,
    terminal::{self, supports_keyboard_enhancement},
};
use crate::{event::InputEvent, port::InputPort};

const POLL_INTERVAL: Duration = Duration::from_millis(20);

// Before the first auto-repeat arrives we can't tell "still held" from "just
// released". Windows' initial repeat delay is ~500ms, so we must wait at least
// that long before firing a synthetic KeyUp.
const INITIAL_HOLD_TIMEOUT: Duration = Duration::from_millis(700);

// Once auto-repeat has started (~30ms between repeats) we know the exact last
// event time. A 150ms gap with no further repeat means the key was released.
const REPEAT_RELEASE_TIMEOUT: Duration = Duration::from_millis(150);

struct KeyState {
    first_press: Instant,
    last_seen:   Instant,
    repeating:   bool, // true after the first auto-repeat arrives
}

pub struct CrosstermInput {
    // True when the terminal confirmed Kitty protocol support and will send
    // genuine KeyEventKind::Release events. Skips synthetic timers entirely.
    enhanced:   bool,
    key_states: HashMap<char, KeyState>,
}

impl CrosstermInput {
    pub fn new() -> anyhow::Result<Self> {
        terminal::enable_raw_mode()?;

        // Query first; always push regardless so borderline terminals pick it up.
        let enhanced = supports_keyboard_enhancement().unwrap_or(false);
        let _ = execute!(
            io::stdout(),
            PushKeyboardEnhancementFlags(
                KeyboardEnhancementFlags::REPORT_EVENT_TYPES
                    | KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES,
            )
        );

        Ok(Self { enhanced, key_states: HashMap::new() })
    }
}

impl Drop for CrosstermInput {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), PopKeyboardEnhancementFlags);
        let _ = terminal::disable_raw_mode();
    }
}

impl InputPort for CrosstermInput {
    fn next_event(&mut self) -> InputEvent {
        loop {
            // ── Two-phase synthetic release (non-enhanced terminals only) ──────
            //
            // Phase 1 — initial hold: no auto-repeat seen yet.
            //   Can't tell released-early from held-waiting-for-first-repeat.
            //   Use INITIAL_HOLD_TIMEOUT (700ms) to survive the Windows
            //   ~500ms initial repeat delay without false-firing.
            //
            // Phase 2 — repeating: auto-repeat confirmed (~30ms intervals).
            //   last_seen is accurate; REPEAT_RELEASE_TIMEOUT (150ms) gives
            //   snappy release while outlasting one missed repeat packet.
            if !self.enhanced {
                let now = Instant::now();
                let stale = self.key_states.iter().find(|(_, s)| {
                    if s.repeating {
                        now.duration_since(s.last_seen) >= REPEAT_RELEASE_TIMEOUT
                    } else {
                        now.duration_since(s.first_press) >= INITIAL_HOLD_TIMEOUT
                    }
                }).map(|(&ch, _)| ch);

                if let Some(ch) = stale {
                    self.key_states.remove(&ch);
                    return if ch == ' ' { InputEvent::SustainOff }
                           else         { InputEvent::KeyUp(ch) };
                }
            }

            match event::poll(POLL_INTERVAL) {
                Ok(true) => {
                    match event::read() {
                        Ok(Event::Key(key)) => {
                            if let Some(ev) = self.translate(key) {
                                return ev;
                            }
                        }
                        Ok(Event::Resize(w, h)) => return InputEvent::Resize(w, h),
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}

impl CrosstermInput {
    fn translate(&mut self, key: KeyEvent) -> Option<InputEvent> {
        if key.code == KeyCode::Esc
            || (key.modifiers.contains(KeyModifiers::CONTROL)
                && key.code == KeyCode::Char('c'))
        {
            return Some(InputEvent::Quit);
        }

        let ch = match key.code {
            KeyCode::Char(c) => c,
            _ => return None,
        };

        match key.kind {
            KeyEventKind::Release => {
                // Genuine key-up (enhanced terminal) — immediate and precise.
                self.key_states.remove(&ch);
                if ch == ' ' { Some(InputEvent::SustainOff) }
                else         { Some(InputEvent::KeyUp(ch)) }
            }

            KeyEventKind::Press | KeyEventKind::Repeat => {
                let now = Instant::now();
                if let Some(state) = self.key_states.get_mut(&ch) {
                    // Key is already tracked — this is auto-repeat.
                    state.last_seen = now;
                    state.repeating = true;
                    if ch == ' ' { return None; }
                    return None; // no re-trigger on repeat
                }

                // First press for this key.
                self.key_states.insert(ch, KeyState {
                    first_press: now,
                    last_seen:   now,
                    repeating:   false,
                });
                if ch == ' ' { Some(InputEvent::SustainOn) }
                else         { Some(InputEvent::KeyDown(ch)) }
            }
        }
    }
}

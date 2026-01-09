use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

use crate::chart::NoteType;

pub enum InputEvent {
    Hit(NoteType),
    Quit,
    None,
}

/// Poll for keyboard input with a timeout
pub fn poll_input(timeout: Duration) -> anyhow::Result<InputEvent> {
    if event::poll(timeout)? {
        if let Event::Key(key) = event::read()? {
            return Ok(map_key_event(key));
        }
    }
    Ok(InputEvent::None)
}

fn map_key_event(key: KeyEvent) -> InputEvent {
    match key.code {
        // D or F = Don (center hit)
        KeyCode::Char('d') | KeyCode::Char('D') | KeyCode::Char('f') | KeyCode::Char('F') => {
            InputEvent::Hit(NoteType::Don)
        }
        // J or K = Ka (rim hit)
        KeyCode::Char('j') | KeyCode::Char('J') | KeyCode::Char('k') | KeyCode::Char('K') => {
            InputEvent::Hit(NoteType::Ka)
        }
        // Escape or Q to quit
        KeyCode::Esc => InputEvent::Quit,
        KeyCode::Char('q') | KeyCode::Char('Q') => InputEvent::Quit,
        // Ctrl+C to quit
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => InputEvent::Quit,
        _ => InputEvent::None,
    }
}

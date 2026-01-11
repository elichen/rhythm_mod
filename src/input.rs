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
        // D = Don Left (center hit, left hand)
        KeyCode::Char('d') | KeyCode::Char('D') => InputEvent::Hit(NoteType::DonLeft),
        // F = Don Right (center hit, right hand)
        KeyCode::Char('f') | KeyCode::Char('F') => InputEvent::Hit(NoteType::DonRight),
        // J = Ka Left (rim hit, left hand)
        KeyCode::Char('j') | KeyCode::Char('J') => InputEvent::Hit(NoteType::KaLeft),
        // K = Ka Right (rim hit, right hand)
        KeyCode::Char('k') | KeyCode::Char('K') => InputEvent::Hit(NoteType::KaRight),
        // Escape or Q to quit
        KeyCode::Esc => InputEvent::Quit,
        KeyCode::Char('q') | KeyCode::Char('Q') => InputEvent::Quit,
        // Ctrl+C to quit
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => InputEvent::Quit,
        _ => InputEvent::None,
    }
}

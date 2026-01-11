#[cfg(test)]
mod ascii_check;
mod audio;
mod chart;
mod game;
mod input;
mod menu;
mod mod_parser;
mod render;

use std::io::{self, stdout};
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

use audio::Audio;
use chart::{Chart, Difficulty};
use game::{Game, HitResult};
use input::{poll_input, InputEvent};
use menu::{handle_menu_input, render_menu, Menu, MenuAction};

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize audio
    let mut audio = Audio::new().ok();

    // Main loop with menu
    let result = run_app(&mut terminal, &mut audio);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    audio: &mut Option<Audio>,
) -> Result<()> {
    let assets_dir = PathBuf::from("assets");

    loop {
        // Show song selection menu
        let mut menu = Menu::new(&assets_dir);

        loop {
            terminal.draw(|f| render_menu(f, &mut menu))?;

            match handle_menu_input(&mut menu)? {
                MenuAction::Select(path, difficulty) => {
                    // Play the selected song
                    if let Err(e) = play_song(terminal, audio, &path, difficulty) {
                        eprintln!("Error playing song: {}", e);
                    }
                    break; // Return to menu after song ends
                }
                MenuAction::Quit => return Ok(()),
                MenuAction::None => {}
            }
        }
    }
}

fn play_song(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    audio: &mut Option<Audio>,
    mod_path: &Path,
    difficulty: Difficulty,
) -> Result<()> {
    // Get the song name for the chart title
    let song_name = mod_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    // Parse chart BEFORE starting audio (so parsing time doesn't cause desync)
    let chart = match Chart::from_mod_file(mod_path, &song_name, difficulty) {
        Ok(c) => c,
        Err(_) => Chart::generic(&song_name, 120.0), // Fallback to generic
    };
    let mut game = Game::new(chart);

    // Start audio and game timer together for proper sync
    if let Some(audio) = audio {
        if let Err(e) = audio.play_mod_file(mod_path) {
            eprintln!("Warning: Could not play MOD file: {}", e);
        }
    }
    game.start(); // Reset timer to NOW, right after audio starts

    // Game loop
    run_game(terminal, &mut game, audio.as_ref())
}

fn run_game(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    game: &mut Game,
    audio: Option<&Audio>,
) -> Result<()> {
    loop {
        // Check for missed notes
        game.check_missed_notes();

        // Update result display
        game.update_result_display();

        // Update equalizer animation
        game.update_equalizer();

        // Render
        terminal.draw(|f| render::render(f, game))?;

        // Check if game is complete
        if game.is_complete() {
            show_results(terminal, game)?;
            break;
        }

        // Poll input (non-blocking, ~60fps)
        match poll_input(Duration::from_millis(16))? {
            InputEvent::Hit(note_type) => {
                let result = game.process_hit(note_type);

                // Play sound
                if let Some(audio) = audio {
                    match result {
                        HitResult::Perfect | HitResult::Good => audio.play_hit(note_type),
                        HitResult::Miss | HitResult::Wrong => audio.play_miss(),
                    }
                }
            }
            InputEvent::Quit => break,
            InputEvent::None => {}
        }
    }

    Ok(())
}

fn show_results(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    game: &Game,
) -> Result<()> {
    // Show results screen
    terminal.draw(|f| render::render_results(f, game))?;

    // Wait for any key
    loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char(' ') | KeyCode::Enter => {
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

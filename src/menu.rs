use std::fs;
use std::path::{Path, PathBuf};

use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::chart::Difficulty;

pub struct SongEntry {
    pub path: PathBuf,
    pub name: String,
}

pub struct Menu {
    pub songs: Vec<SongEntry>,
    pub selected: usize,
    pub list_state: ListState,
    pub difficulty: Difficulty,
}

impl Menu {
    pub fn new(assets_dir: &Path) -> Self {
        let mut songs = Vec::new();

        // Scan for MOD files
        if let Ok(entries) = fs::read_dir(assets_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext.to_string_lossy().to_lowercase() == "mod" {
                        let name = path
                            .file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| "Unknown".to_string());
                        songs.push(SongEntry { path, name });
                    }
                }
            }
        }

        // Sort by name
        songs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        let mut list_state = ListState::default();
        if !songs.is_empty() {
            list_state.select(Some(0));
        }

        Menu {
            songs,
            selected: 0,
            list_state,
            difficulty: Difficulty::default(),
        }
    }

    pub fn cycle_difficulty(&mut self) {
        self.difficulty = match self.difficulty {
            Difficulty::Easy => Difficulty::Medium,
            Difficulty::Medium => Difficulty::Hard,
            Difficulty::Hard => Difficulty::Easy,
        };
    }

    pub fn up(&mut self) {
        if !self.songs.is_empty() && self.selected > 0 {
            self.selected -= 1;
            self.list_state.select(Some(self.selected));
        }
    }

    pub fn down(&mut self) {
        if !self.songs.is_empty() && self.selected < self.songs.len() - 1 {
            self.selected += 1;
            self.list_state.select(Some(self.selected));
        }
    }

    pub fn selected_song(&self) -> Option<&SongEntry> {
        self.songs.get(self.selected)
    }

    pub fn is_empty(&self) -> bool {
        self.songs.is_empty()
    }
}

pub fn render_menu(frame: &mut Frame, menu: &mut Menu) {
    let size = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Header
            Constraint::Min(10),   // Song list
            Constraint::Length(3), // Footer
        ])
        .split(size);

    // Header with difficulty
    let difficulty_color = match menu.difficulty {
        Difficulty::Easy => Color::Green,
        Difficulty::Medium => Color::Yellow,
        Difficulty::Hard => Color::Red,
    };
    let title = vec![
        Line::from(""),
        Line::from(Span::styled(
            "TAIKO TERMINAL",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("Difficulty: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("< {} >", menu.difficulty.name()),
                Style::default().fg(difficulty_color).add_modifier(Modifier::BOLD),
            ),
        ]),
    ];
    let header = Paragraph::new(title)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));
    frame.render_widget(header, chunks[0]);

    // Song list
    if menu.is_empty() {
        let no_songs = Paragraph::new(vec![
            Line::from(""),
            Line::from("No MOD files found in assets/ folder"),
            Line::from(""),
            Line::from("Place .mod files in the assets/ directory"),
        ])
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Red));
        frame.render_widget(no_songs, chunks[1]);
    } else {
        let items: Vec<ListItem> = menu
            .songs
            .iter()
            .enumerate()
            .map(|(i, song)| {
                let style = if i == menu.selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                let prefix = if i == menu.selected { "> " } else { "  " };
                ListItem::new(format!("{}{}", prefix, song.name)).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Songs ")
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_stateful_widget(list, chunks[1], &mut menu.list_state);
    }

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("[Up/Down]", Style::default().fg(Color::Cyan)),
        Span::raw(" Song  "),
        Span::styled("[Left/Right]", Style::default().fg(Color::Cyan)),
        Span::raw(" Difficulty  "),
        Span::styled("[Enter]", Style::default().fg(Color::Cyan)),
        Span::raw(" Play  "),
        Span::styled("[Esc]", Style::default().fg(Color::Cyan)),
        Span::raw(" Quit"),
    ]))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::TOP));
    frame.render_widget(footer, chunks[2]);
}

pub enum MenuAction {
    Select(PathBuf, Difficulty),
    Quit,
    None,
}

pub fn handle_menu_input(menu: &mut Menu) -> anyhow::Result<MenuAction> {
    if event::poll(std::time::Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => menu.up(),
                KeyCode::Down | KeyCode::Char('j') => menu.down(),
                KeyCode::Left | KeyCode::Right | KeyCode::Char('h') | KeyCode::Char('l') => {
                    menu.cycle_difficulty();
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    if let Some(song) = menu.selected_song() {
                        return Ok(MenuAction::Select(song.path.clone(), menu.difficulty));
                    }
                }
                KeyCode::Esc | KeyCode::Char('q') => return Ok(MenuAction::Quit),
                _ => {}
            }
        }
    }
    Ok(MenuAction::None)
}

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::chart::NoteType;
use crate::game::{Game, HitResult};

const SCROLL_WINDOW_MS: u64 = 2200;
const NOTE_WIDTH: usize = 5;

// Color palette - using ANSI colors for macOS Terminal.app compatibility
// (Terminal.app doesn't support true color/RGB - only 16 ANSI + 256 indexed)
const COLOR_DON: Color = Color::Red;                   // Red for DON drums
const COLOR_KA: Color = Color::Cyan;                   // Cyan for KA drums
const COLOR_GOLD: Color = Color::Yellow;              // Yellow accent
const COLOR_TRACK: Color = Color::DarkGray;           // Track background
const COLOR_DIM: Color = Color::DarkGray;             // Border color
const COLOR_BG: Color = Color::Black;                 // Explicit background
const COLOR_BG_ACCENT: Color = Color::Black;          // Dark accent (same as bg for compatibility)

// Equalizer channel colors - using ANSI colors for compatibility
const EQ_COLORS: [Color; 4] = [
    Color::LightRed,            // CH1 - Red
    Color::LightGreen,          // CH2 - Green
    Color::LightBlue,           // CH3 - Blue
    Color::LightYellow,         // CH4 - Yellow
];

pub fn render(frame: &mut Frame, game: &Game) {
    let size = frame.area();

    // Clear background with explicit black to ensure consistent colors on macOS Terminal
    frame.render_widget(
        Block::default().style(Style::default().bg(COLOR_BG)),
        size,
    );

    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),  // Header
            Constraint::Length(9),  // Game area
            Constraint::Length(9),  // Equalizer
            Constraint::Min(3),     // Feedback
            Constraint::Length(2),  // Footer
        ])
        .split(size);

    render_header(frame, chunks[0], game);
    render_game_area(frame, chunks[1], game);
    render_equalizer(frame, chunks[2], game);
    render_feedback(frame, chunks[3], game);
    render_footer(frame, chunks[4]);
}

fn render_header(frame: &mut Frame, area: Rect, game: &Game) {
    // Calculate soul percentage
    let total = game.chart.notes.len() as f64;
    let hit = (game.perfect_count + game.good_count) as f64;
    let soul_pct = if total > 0.0 { (hit / total * 100.0).min(100.0) } else { 0.0 };

    // Build custom gauge with block characters
    let gauge_width = 20;
    let filled = ((soul_pct / 100.0) * gauge_width as f64) as usize;

    let gauge_color = if soul_pct >= 80.0 {
        COLOR_GOLD
    } else if soul_pct >= 50.0 {
        COLOR_DON
    } else {
        Color::White
    };

    // Top border (use spans for consistent styling)
    let border_top = "─".repeat(area.width.saturating_sub(2) as usize);
    let top_border = Line::from(vec![
        Span::styled("┌", Style::default().fg(COLOR_DIM)),
        Span::styled(&border_top, Style::default().fg(COLOR_DIM)),
        Span::styled("┐", Style::default().fg(COLOR_DIM)),
    ]);
    frame.render_widget(
        Paragraph::new(top_border),
        Rect::new(area.x, area.y, area.width, 1),
    );

    // Title line
    let title = format!(" ◆ {} ", game.chart.title);
    let score_str = format!("{:08}", game.score);

    // Calculate: │(1) + ` ▄▀▀▀▄ `(7) + title + padding + score(8) + ` │`(2) = width
    // Note: title.len() is bytes, but display width differs for Unicode like ◆
    // title format is " ◆ {} " = space(1) + ◆(1) + space(1) + title + space(1) = 4 + title chars
    let title_display_width = 4 + game.chart.title.chars().count();
    let title_content = 1 + 7 + title_display_width + score_str.len() + 2;
    let title_padding = (area.width as usize).saturating_sub(title_content);

    let title_spans = vec![
        Span::styled("│", Style::default().fg(COLOR_DIM)),
        Span::styled(" ▄▀▀▀▄ ", Style::default().fg(COLOR_GOLD).add_modifier(Modifier::BOLD)),
        Span::styled(&title, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::raw(" ".repeat(title_padding)),
        Span::styled(&score_str, Style::default().fg(COLOR_GOLD).add_modifier(Modifier::BOLD)),
        Span::styled(" │", Style::default().fg(COLOR_DIM)),
    ];

    frame.render_widget(
        Paragraph::new(Line::from(title_spans)),
        Rect::new(area.x, area.y + 1, area.width, 1),
    );

    // Soul gauge line
    // Calculate: │(1) + ` SOUL `(6) + ▐(1) + gauge(20) + ▌(1) + ` NNN%`(5) + padding + │(1) = width
    // So: padding = width - 35
    let gauge_full: String = "█".repeat(filled);
    let gauge_empty: String = "░".repeat(gauge_width - filled);
    let pct_str = format!(" {:3.0}%", soul_pct);
    let gauge_content = 1 + 6 + 1 + gauge_width + 1 + pct_str.len() + 1; // 35
    let gauge_padding = (area.width as usize).saturating_sub(gauge_content);

    let gauge_spans = vec![
        Span::styled("│", Style::default().fg(COLOR_DIM)),
        Span::styled(" SOUL ", Style::default().fg(Color::White)),
        Span::styled("▐", Style::default().fg(gauge_color)),
        Span::styled(&gauge_full, Style::default().fg(gauge_color)),
        Span::styled(&gauge_empty, Style::default().fg(COLOR_TRACK)),
        Span::styled("▌", Style::default().fg(gauge_color)),
        Span::styled(pct_str, Style::default().fg(Color::White)),
        Span::raw(" ".repeat(gauge_padding)),
        Span::styled("│", Style::default().fg(COLOR_DIM)),
    ];

    frame.render_widget(
        Paragraph::new(Line::from(gauge_spans)),
        Rect::new(area.x, area.y + 2, area.width, 1),
    );

    // Bottom border (use spans for consistent styling)
    let bottom_border = Line::from(vec![
        Span::styled("├", Style::default().fg(COLOR_DIM)),
        Span::styled(&border_top, Style::default().fg(COLOR_DIM)),
        Span::styled("┤", Style::default().fg(COLOR_DIM)),
    ]);
    frame.render_widget(
        Paragraph::new(bottom_border),
        Rect::new(area.x, area.y + 3, area.width, 1),
    );
}

fn render_game_area(frame: &mut Frame, area: Rect, game: &Game) {
    if area.width < 50 || area.height < 9 {
        return;
    }

    let current_time = game.current_time_ms();

    // The drum target - bold geometric design
    // 7 lines tall, centered vertically
    let (drum_color, drum_glow) = match game.last_hit_result {
        Some(HitResult::Perfect) => (COLOR_GOLD, Color::Yellow),
        Some(HitResult::Good) => (Color::Green, Color::LightGreen),
        Some(HitResult::Miss) | Some(HitResult::Wrong) => (Color::Red, Color::LightRed),
        None => (Color::White, COLOR_DIM),
    };

    let drum_art = [
        ("   ▄▄███▄▄    ", drum_glow),
        ("  █▓▓▓▓▓▓▓█   ", drum_color),
        (" █▓▓█████▓▓█  ", drum_color),
        (" █▓██   ██▓█  ", drum_color),
        (" █▓▓█████▓▓█  ", drum_color),
        ("  █▓▓▓▓▓▓▓█   ", drum_color),
        ("   ▀▀███▀▀    ", drum_glow),
    ];

    let drum_width = 14;
    let track_start = drum_width + 3;
    let track_width = (area.width as usize).saturating_sub(track_start + 3);

    // Collect visible notes
    let mut notes: Vec<(usize, NoteType)> = Vec::new();
    for note in &game.chart.notes {
        if note.hit { continue; }

        let time_diff = note.time_ms as i64 - current_time as i64;
        if time_diff < -100 || time_diff > SCROLL_WINDOW_MS as i64 { continue; }

        let progress = time_diff as f64 / SCROLL_WINDOW_MS as f64;
        let x = (track_width as f64 * progress) as usize;

        if x + NOTE_WIDTH < track_width {
            notes.push((x, note.note_type));
        }
    }
    notes.sort_by_key(|(x, _)| *x);

    // Render each row
    for row in 0..7 {
        let mut spans: Vec<Span> = Vec::new();

        // Left margin
        spans.push(Span::styled("│ ", Style::default().fg(COLOR_DIM)));

        // Drum target
        spans.push(Span::styled(drum_art[row].0, Style::default().fg(drum_art[row].1)));

        // Hit zone separator
        spans.push(Span::styled(" ┃", Style::default().fg(COLOR_GOLD).add_modifier(Modifier::BOLD)));

        // Track with notes (only on rows 2-4, the middle band)
        if row >= 2 && row <= 4 {
            let track_row = row - 2; // 0, 1, or 2
            let mut x = 0;

            while x < track_width {
                // Check for note at this position
                let note_at_x = notes.iter().find(|(nx, _)| *nx <= x && x < *nx + NOTE_WIDTH);

                if let Some((_, note_type)) = note_at_x {
                    // Draw note segment
                    let (chars, color) = match (note_type, track_row) {
                        // DON LEFT - solid with D
                        (NoteType::DonLeft, 0) => ("▄███▄", COLOR_DON),
                        (NoteType::DonLeft, 1) => ("█ D █", COLOR_DON),
                        (NoteType::DonLeft, 2) => ("▀███▀", COLOR_DON),
                        // DON RIGHT - solid with F
                        (NoteType::DonRight, 0) => ("▄███▄", COLOR_DON),
                        (NoteType::DonRight, 1) => ("█ F █", COLOR_DON),
                        (NoteType::DonRight, 2) => ("▀███▀", COLOR_DON),
                        // KA LEFT - hollow with J
                        (NoteType::KaLeft, 0) => ("▄▀▀▀▄", COLOR_KA),
                        (NoteType::KaLeft, 1) => ("█ J █", COLOR_KA),
                        (NoteType::KaLeft, 2) => ("▀▄▄▄▀", COLOR_KA),
                        // KA RIGHT - hollow with K
                        (NoteType::KaRight, 0) => ("▄▀▀▀▄", COLOR_KA),
                        (NoteType::KaRight, 1) => ("█ K █", COLOR_KA),
                        (NoteType::KaRight, 2) => ("▀▄▄▄▀", COLOR_KA),
                        _ => ("     ", COLOR_TRACK),
                    };
                    spans.push(Span::styled(chars, Style::default().fg(color).add_modifier(Modifier::BOLD)));
                    x += NOTE_WIDTH;
                } else {
                    // Draw track
                    let track_char = if track_row == 1 { "═" } else { "─" };
                    spans.push(Span::styled(track_char, Style::default().fg(COLOR_TRACK)));
                    x += 1;
                }
            }
        } else {
            // Empty track rows (decorative lines above/below)
            if row == 1 || row == 5 {
                let deco = "░".repeat(track_width);
                spans.push(Span::styled(deco, Style::default().fg(COLOR_BG_ACCENT)));
            } else {
                spans.push(Span::raw(" ".repeat(track_width)));
            }
        }

        // Right margin
        spans.push(Span::styled(" │", Style::default().fg(COLOR_DIM)));

        frame.render_widget(
            Paragraph::new(Line::from(spans)),
            Rect::new(area.x, area.y + row as u16 + 1, area.width, 1),
        );
    }

    // Top and bottom of game area (use spans for consistent styling)
    let border = "─".repeat(area.width.saturating_sub(2) as usize);
    let inner_width = area.width.saturating_sub(2) as usize;
    let game_top = Line::from(vec![
        Span::styled("│", Style::default().fg(COLOR_DIM)),
        Span::raw(" ".repeat(inner_width)),
        Span::styled("│", Style::default().fg(COLOR_DIM)),
    ]);
    frame.render_widget(
        Paragraph::new(game_top),
        Rect::new(area.x, area.y, area.width, 1),
    );
    let game_bottom = Line::from(vec![
        Span::styled("├", Style::default().fg(COLOR_DIM)),
        Span::styled(&border, Style::default().fg(COLOR_DIM)),
        Span::styled("┤", Style::default().fg(COLOR_DIM)),
    ]);
    frame.render_widget(
        Paragraph::new(game_bottom),
        Rect::new(area.x, area.y + 8, area.width, 1),
    );
}

fn render_equalizer(frame: &mut Frame, area: Rect, game: &Game) {
    // Bold retro tracker style - VU meter gradient (green → yellow → red)
    let bar_height = 7;

    // VU meter color gradient (bottom to top)
    let vu_colors: [Color; 7] = [
        Color::Rgb(0, 255, 100),    // Bottom - bright green
        Color::Rgb(50, 255, 50),    // Green
        Color::Rgb(150, 255, 0),    // Yellow-green
        Color::Rgb(255, 255, 0),    // Yellow
        Color::Rgb(255, 200, 0),    // Orange-yellow
        Color::Rgb(255, 100, 0),    // Orange
        Color::Rgb(255, 0, 50),     // Top - hot red (danger zone!)
    ];

    // Calculate centering: 4 channels × 12 chars (▐ + 10 + ▌) + 3 spaces between = 51 chars
    let content_width = 51;
    let available = area.width.saturating_sub(2) as usize; // minus borders
    let pad_left = available.saturating_sub(content_width) / 2;
    let pad_right = available.saturating_sub(content_width + pad_left);

    // Title: │ + pad_left + ▄▀(2) + title(27 display width) + ▀▄(2) + pad_right + │
    // Note: title.len() returns bytes (31), but display width is 27
    let title = " ◆ M O D   T R A C K E R ◆ ";
    let title_display_width = 27; // Actual display width (◆ is 1 width but 3 bytes)
    let title_content_width = 2 + title_display_width + 2; // ▄▀ + title + ▀▄ = 31 display
    let title_pad = available.saturating_sub(title_content_width) / 2;
    let title_pad_right = available.saturating_sub(title_pad + title_content_width);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("│", Style::default().fg(COLOR_DIM)),
            Span::raw(" ".repeat(title_pad)),
            Span::styled("▄▀", Style::default().fg(COLOR_GOLD)),
            Span::styled(title, Style::default().fg(COLOR_GOLD).add_modifier(Modifier::BOLD)),
            Span::styled("▀▄", Style::default().fg(COLOR_GOLD)),
            Span::raw(" ".repeat(title_pad_right)),
            Span::styled("│", Style::default().fg(COLOR_DIM)),
        ])),
        Rect::new(area.x, area.y, area.width, 1),
    );

    // Render the 7 rows of bars (top = loud, bottom = quiet)
    for row in 0..bar_height {
        let threshold = 1.0 - (row as f32 / bar_height as f32);
        let row_color = vu_colors[row];
        let dim_color = Color::Rgb(30, 30, 40);

        let mut row_spans: Vec<Span> = vec![
            Span::styled("│", Style::default().fg(COLOR_DIM)),
            Span::raw(" ".repeat(pad_left)),
        ];

        for ch in 0..4 {
            let level = game.eq_levels[ch];

            // Left bracket
            row_spans.push(Span::styled("▐", Style::default().fg(EQ_COLORS[ch])));

            // The bar content - 10 chars wide for BOLD presence
            let (block, fg, is_peak) = if level >= threshold {
                if row == 0 && level > 0.85 {
                    // PEAK! Flashing/bright
                    ("██████████", Color::White, true)
                } else {
                    ("██████████", row_color, false)
                }
            } else if level >= threshold - 0.08 {
                // Edge of the bar - partial fill
                ("▄▄▄▄▄▄▄▄▄▄", row_color, false)
            } else {
                // Empty segment - dark with grid lines
                if row == 3 {
                    ("──────────", dim_color, false)
                } else {
                    ("          ", dim_color, false)
                }
            };

            let style = if is_peak {
                Style::default().fg(fg).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(fg)
            };
            row_spans.push(Span::styled(block, style));

            // Right bracket
            row_spans.push(Span::styled("▌", Style::default().fg(EQ_COLORS[ch])));

            // Spacing between channels
            if ch < 3 {
                row_spans.push(Span::raw(" "));
            }
        }

        row_spans.push(Span::raw(" ".repeat(pad_right)));
        row_spans.push(Span::styled("│", Style::default().fg(COLOR_DIM)));

        frame.render_widget(
            Paragraph::new(Line::from(row_spans)),
            Rect::new(area.x, area.y + 1 + row as u16, area.width, 1),
        );
    }

    // Channel labels at bottom - centered
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("│", Style::default().fg(COLOR_DIM)),
            Span::raw(" ".repeat(pad_left)),
            Span::styled("▌", Style::default().fg(EQ_COLORS[0])),
            Span::styled("  CHAN 1  ", Style::default().fg(EQ_COLORS[0]).add_modifier(Modifier::BOLD)),
            Span::styled("▐", Style::default().fg(EQ_COLORS[0])),
            Span::raw(" "),
            Span::styled("▌", Style::default().fg(EQ_COLORS[1])),
            Span::styled("  CHAN 2  ", Style::default().fg(EQ_COLORS[1]).add_modifier(Modifier::BOLD)),
            Span::styled("▐", Style::default().fg(EQ_COLORS[1])),
            Span::raw(" "),
            Span::styled("▌", Style::default().fg(EQ_COLORS[2])),
            Span::styled("  CHAN 3  ", Style::default().fg(EQ_COLORS[2]).add_modifier(Modifier::BOLD)),
            Span::styled("▐", Style::default().fg(EQ_COLORS[2])),
            Span::raw(" "),
            Span::styled("▌", Style::default().fg(EQ_COLORS[3])),
            Span::styled("  CHAN 4  ", Style::default().fg(EQ_COLORS[3]).add_modifier(Modifier::BOLD)),
            Span::styled("▐", Style::default().fg(EQ_COLORS[3])),
            Span::raw(" ".repeat(pad_right)),
            Span::styled("│", Style::default().fg(COLOR_DIM)),
        ])),
        Rect::new(area.x, area.y + 8, area.width, 1),
    );
}

fn render_feedback(frame: &mut Frame, area: Rect, game: &Game) {
    let inner_width = area.width.saturating_sub(2) as usize;

    // Render each row with borders
    for row in 0..area.height {
        let line = if row == 1 {
            // Hit result row
            if let Some(result) = game.last_hit_result {
                let (text, color) = match result {
                    HitResult::Perfect => ("  ★ ★ ★  P E R F E C T  ★ ★ ★  ", COLOR_GOLD),
                    HitResult::Good => ("  ★  G O O D  ★  ", Color::Green),
                    HitResult::Miss => ("  ×  M I S S  ×  ", Color::Red),
                    HitResult::Wrong => ("  ×  W R O N G  ×  ", Color::Magenta),
                };
                let content_width = text.chars().count();
                let pad_left = inner_width.saturating_sub(content_width) / 2;
                let pad_right = inner_width.saturating_sub(content_width + pad_left);
                Line::from(vec![
                    Span::styled("│", Style::default().fg(COLOR_DIM)),
                    Span::raw(" ".repeat(pad_left)),
                    Span::styled(text, Style::default().fg(color).add_modifier(Modifier::BOLD)),
                    Span::raw(" ".repeat(pad_right)),
                    Span::styled("│", Style::default().fg(COLOR_DIM)),
                ])
            } else {
                Line::from(vec![
                    Span::styled("│", Style::default().fg(COLOR_DIM)),
                    Span::raw(" ".repeat(inner_width)),
                    Span::styled("│", Style::default().fg(COLOR_DIM)),
                ])
            }
        } else if row == 3 && game.combo >= 2 {
            // Combo row
            let combo_color = if game.combo >= 100 {
                COLOR_GOLD
            } else if game.combo >= 50 {
                Color::Magenta
            } else if game.combo >= 10 {
                COLOR_KA
            } else {
                Color::White
            };
            let combo_text = if game.combo >= 50 {
                format!("▶▶▶  {}  C O M B O  ◀◀◀", game.combo)
            } else {
                format!("{}  C O M B O", game.combo)
            };
            let content_width = combo_text.chars().count();
            let pad_left = inner_width.saturating_sub(content_width) / 2;
            let pad_right = inner_width.saturating_sub(content_width + pad_left);
            Line::from(vec![
                Span::styled("│", Style::default().fg(COLOR_DIM)),
                Span::raw(" ".repeat(pad_left)),
                Span::styled(combo_text, Style::default().fg(combo_color).add_modifier(Modifier::BOLD)),
                Span::raw(" ".repeat(pad_right)),
                Span::styled("│", Style::default().fg(COLOR_DIM)),
            ])
        } else {
            // Empty bordered line
            Line::from(vec![
                Span::styled("│", Style::default().fg(COLOR_DIM)),
                Span::raw(" ".repeat(inner_width)),
                Span::styled("│", Style::default().fg(COLOR_DIM)),
            ])
        };

        frame.render_widget(
            Paragraph::new(line),
            Rect::new(area.x, area.y + row, area.width, 1),
        );
    }
}

fn render_footer(frame: &mut Frame, area: Rect) {
    // Calculate content width: "███ DON [D][F]        █◆█ KA [J][K]        [ESC] quit"
    // = 3 + 5 + 6 + 8 + 3 + 4 + 6 + 8 + 5 + 5 = 53 chars
    let content_width = 53;
    let available = area.width.saturating_sub(2) as usize; // minus borders
    let pad_left = available.saturating_sub(content_width) / 2;
    let pad_right = available.saturating_sub(content_width + pad_left);

    let footer = Line::from(vec![
        Span::styled("│", Style::default().fg(COLOR_DIM)),
        Span::raw(" ".repeat(pad_left)),
        Span::styled("███", Style::default().fg(COLOR_DON)),
        Span::styled(" DON ", Style::default().fg(COLOR_DON).add_modifier(Modifier::BOLD)),
        Span::styled("[D][F]", Style::default().fg(Color::White)),
        Span::raw("        "),
        Span::styled("█◆█", Style::default().fg(COLOR_KA)),
        Span::styled(" KA ", Style::default().fg(COLOR_KA).add_modifier(Modifier::BOLD)),
        Span::styled("[J][K]", Style::default().fg(Color::White)),
        Span::raw("        "),
        Span::styled("[ESC]", Style::default().fg(Color::DarkGray)),
        Span::styled(" quit", Style::default().fg(Color::DarkGray)),
        Span::raw(" ".repeat(pad_right)),
        Span::styled("│", Style::default().fg(COLOR_DIM)),
    ]);

    frame.render_widget(
        Paragraph::new(footer),
        Rect::new(area.x, area.y, area.width, 1),
    );

    // Bottom border (use spans for consistent styling)
    let border = "─".repeat(area.width.saturating_sub(2) as usize);
    let bottom_border = Line::from(vec![
        Span::styled("└", Style::default().fg(COLOR_DIM)),
        Span::styled(&border, Style::default().fg(COLOR_DIM)),
        Span::styled("┘", Style::default().fg(COLOR_DIM)),
    ]);
    frame.render_widget(
        Paragraph::new(bottom_border),
        Rect::new(area.x, area.y + 1, area.width, 1),
    );
}

pub fn render_results(frame: &mut Frame, game: &Game) {
    let size = frame.area();

    let accuracy = if game.perfect_count + game.good_count + game.miss_count > 0 {
        let total = game.perfect_count + game.good_count + game.miss_count;
        ((game.perfect_count as f64 * 100.0 + game.good_count as f64 * 50.0) / total as f64) as u32
    } else {
        0
    };

    let (grade, grade_color) = match accuracy {
        95..=100 => ("S", COLOR_GOLD),
        85..=94 => ("A", Color::Magenta),
        70..=84 => ("B", COLOR_KA),
        50..=69 => ("C", Color::Green),
        _ => ("D", Color::Red),
    };

    // Big ASCII grade
    let grade_art: Vec<&str> = match grade {
        "S" => vec![
            "  ▄▄▄▄▄▄▄   ",
            " ██▀▀▀▀▀    ",
            " ▀██████▄   ",
            "      ▀▀██  ",
            " ▄▄▄▄▄██▀   ",
        ],
        "A" => vec![
            "    ▄██▄    ",
            "   ██  ██   ",
            "  ████████  ",
            " ██      ██ ",
            " ██      ██ ",
        ],
        "B" => vec![
            " ██████▄    ",
            " ██   ▀██   ",
            " ██████▀    ",
            " ██   ▄██   ",
            " ██████▀    ",
        ],
        "C" => vec![
            "  ▄█████▄   ",
            " ██▀        ",
            " ██         ",
            " ██▄        ",
            "  ▀█████▀   ",
        ],
        _ => vec![
            " ██████▄    ",
            " ██   ▀██   ",
            " ██    ██   ",
            " ██   ▄██   ",
            " ██████▀    ",
        ],
    };

    let mut lines: Vec<Line> = Vec::new();

    // Title
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "═══════════════════════════════════",
        Style::default().fg(COLOR_GOLD),
    )));
    lines.push(Line::from(Span::styled(
        "           R E S U L T S           ",
        Style::default().fg(COLOR_GOLD).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        "═══════════════════════════════════",
        Style::default().fg(COLOR_GOLD),
    )));
    lines.push(Line::from(""));

    // Grade art
    for line in grade_art {
        lines.push(Line::from(Span::styled(
            format!("          {}", line),
            Style::default().fg(grade_color).add_modifier(Modifier::BOLD),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("       SCORE    {:08}", game.score),
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("       PERFECT  ", Style::default().fg(COLOR_GOLD)),
        Span::styled(format!("{:4}", game.perfect_count), Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("       GOOD     ", Style::default().fg(Color::Green)),
        Span::styled(format!("{:4}", game.good_count), Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("       MISS     ", Style::default().fg(Color::Red)),
        Span::styled(format!("{:4}", game.miss_count), Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("       MAX COMBO   {}", game.max_combo),
        Style::default().fg(Color::Magenta),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "───────────────────────────────────",
        Style::default().fg(COLOR_DIM),
    )));
    lines.push(Line::from(Span::styled(
        "     Press ENTER to continue...    ",
        Style::default().fg(Color::DarkGray),
    )));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(grade_color));

    let inner = block.inner(size);
    frame.render_widget(block, size);
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), inner);
}

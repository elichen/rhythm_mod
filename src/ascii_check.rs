// ASCII art width/height consistency checker
// Run with: cargo test ascii_check -- --nocapture

use unicode_width::UnicodeWidthStr;

/// Check that all strings in a slice have the same display width
fn check_widths(name: &str, art: &[&str]) {
    println!("\n=== {} ===", name);
    let mut widths: Vec<usize> = Vec::new();

    for (i, line) in art.iter().enumerate() {
        let char_count = line.chars().count();
        let display_width = UnicodeWidthStr::width(*line);
        widths.push(display_width);
        println!(
            "  Line {}: chars={:2}, display_width={:2}, content={:?}",
            i, char_count, display_width, line
        );
    }

    let first_width = widths[0];
    let all_same = widths.iter().all(|&w| w == first_width);

    if all_same {
        println!("  ✓ All lines have consistent width: {}", first_width);
    } else {
        println!("  ✗ INCONSISTENT WIDTHS: {:?}", widths);
        for (i, &w) in widths.iter().enumerate() {
            if w != first_width {
                println!("    Line {} has width {} (expected {})", i, w, first_width);
            }
        }
    }
}

/// Check that a string has the expected display width
fn check_single(name: &str, s: &str, expected: usize) {
    let char_count = s.chars().count();
    let display_width = UnicodeWidthStr::width(s);
    let status = if display_width == expected { "✓" } else { "✗" };
    println!(
        "{} {}: chars={}, display_width={}, expected={}, content={:?}",
        status, name, char_count, display_width, expected, s
    );
}

pub fn run_checks() {
    println!("\n======== ASCII ART CONSISTENCY CHECK ========");

    // ========== DRUM ART (from render.rs line 148-156) ==========
    // Expected: 14 display width (drum_width = 14)
    let drum_art = [
        "   ▄▄███▄▄    ",
        "  █▓▓▓▓▓▓▓█   ",
        " █▓▓█████▓▓█  ",
        " █▓██   ██▓█  ",
        " █▓▓█████▓▓█  ",
        "  █▓▓▓▓▓▓▓█   ",
        "   ▀▀███▀▀    ",
    ];
    check_widths("DRUM ART (expected width: 14)", &drum_art);

    // ========== NOTE PATTERNS (from render.rs line 205-212) ==========
    // Expected: 5 display width (NOTE_WIDTH = 5)
    println!("\n=== NOTE PATTERNS (expected width: 5) ===");

    // DON notes
    check_single("DON row 0", "▄███▄", 5);
    check_single("DON row 1", "█████", 5);
    check_single("DON row 2", "▀███▀", 5);

    // KA notes
    check_single("KA row 0", "▄▀▀▀▄", 5);
    check_single("KA row 1", "█ ◆ █", 5);
    check_single("KA row 2", "▀▄▄▄▀", 5);

    // ========== GRADE ART (from render.rs line 481-517) ==========
    let grade_s = [
        "  ▄▄▄▄▄▄▄   ",
        " ██▀▀▀▀▀    ",
        " ▀██████▄   ",
        "      ▀▀██  ",
        " ▄▄▄▄▄██▀   ",
    ];
    check_widths("GRADE S", &grade_s);

    let grade_a = [
        "    ▄██▄    ",
        "   ██  ██   ",
        "  ████████  ",
        " ██      ██ ",
        " ██      ██ ",
    ];
    check_widths("GRADE A", &grade_a);

    let grade_b = [
        " ██████▄    ",
        " ██   ▀██   ",
        " ██████▀    ",
        " ██   ▄██   ",
        " ██████▀    ",
    ];
    check_widths("GRADE B", &grade_b);

    let grade_c = [
        "  ▄█████▄   ",
        " ██▀        ",
        " ██         ",
        " ██▄        ",
        "  ▀█████▀   ",
    ];
    check_widths("GRADE C", &grade_c);

    let grade_d = [
        " ██████▄    ",
        " ██   ▀██   ",
        " ██    ██   ",
        " ██   ▄██   ",
        " ██████▀    ",
    ];
    check_widths("GRADE D", &grade_d);

    // Check that all grades have the same width
    println!("\n=== GRADE WIDTH COMPARISON ===");
    let grade_widths = [
        ("S", UnicodeWidthStr::width(grade_s[0])),
        ("A", UnicodeWidthStr::width(grade_a[0])),
        ("B", UnicodeWidthStr::width(grade_b[0])),
        ("C", UnicodeWidthStr::width(grade_c[0])),
        ("D", UnicodeWidthStr::width(grade_d[0])),
    ];
    for (name, width) in &grade_widths {
        println!("  Grade {}: width={}", name, width);
    }
    let first_grade_width = grade_widths[0].1;
    if grade_widths.iter().all(|(_, w)| *w == first_grade_width) {
        println!("  ✓ All grades have same width");
    } else {
        println!("  ✗ GRADES HAVE INCONSISTENT WIDTHS");
    }

    // ========== HEADER DECORATIONS ==========
    println!("\n=== HEADER DECORATIONS ===");
    check_single("Header taiko", "▄▀▀▀▄", 5);
    check_single("Gauge left cap", "▐", 1);
    check_single("Gauge right cap", "▌", 1);
    check_single("Gauge full block", "█", 1);
    check_single("Gauge empty block", "░", 1);

    // ========== EQUALIZER ==========
    println!("\n=== EQUALIZER BARS ===");
    check_single("EQ bar full", "██████████", 10);
    check_single("EQ bar partial", "▄▄▄▄▄▄▄▄▄▄", 10);
    check_single("EQ bar grid", "──────────", 10);
    check_single("EQ left bracket", "▐", 1);
    check_single("EQ right bracket", "▌", 1);

    // ========== FEEDBACK TEXT ==========
    println!("\n=== FEEDBACK TEXT ===");
    let perfect_text = "  ★ ★ ★  P E R F E C T  ★ ★ ★  ";
    let good_text = "  ★  G O O D  ★  ";
    let miss_text = "  ×  M I S S  ×  ";
    let wrong_text = "  ×  W R O N G  ×  ";

    check_single("PERFECT", perfect_text, 31);
    check_single("GOOD", good_text, 17);
    check_single("MISS", miss_text, 17);
    check_single("WRONG", wrong_text, 19);

    // ========== FOOTER ==========
    println!("\n=== FOOTER ===");
    check_single("DON block", "███", 3);
    check_single("KA block", "█◆█", 3);

    // ========== BOX BORDER CHARACTERS ==========
    println!("\n=== BOX BORDER CHARACTERS ===");
    // All box-drawing characters should be width 1
    check_single("Corner TL ┌", "┌", 1);
    check_single("Corner TR ┐", "┐", 1);
    check_single("Corner BL └", "└", 1);
    check_single("Corner BR ┘", "┘", 1);
    check_single("T-junction L ├", "├", 1);
    check_single("T-junction R ┤", "┤", 1);
    check_single("Vertical │", "│", 1);
    check_single("Horizontal ─", "─", 1);
    check_single("Double horiz ═", "═", 1);
    check_single("Bold vertical ┃", "┃", 1);

    // ========== BORDER MARGIN PATTERNS ==========
    println!("\n=== BORDER MARGIN PATTERNS ===");
    // Game area uses "│ " (2 chars) for left margin and " │" (2 chars) for right
    check_single("Left margin '│ '", "│ ", 2);
    check_single("Right margin ' │'", " │", 2);
    // Header/equalizer use just "│" (1 char)
    check_single("Simple border '│'", "│", 1);
    // Hit zone separator
    check_single("Hit zone ' ┃'", " ┃", 2);

    // ========== RESULTS SCREEN DECORATIONS ==========
    println!("\n=== RESULTS SCREEN DECORATIONS ===");
    let results_double_line = "═══════════════════════════════════";
    let results_title = "           R E S U L T S           ";
    let results_single_line = "───────────────────────────────────";
    let results_continue = "     Press ENTER to continue...    ";

    check_single("Results double line", results_double_line, 35);
    check_single("Results title", results_title, 35);
    check_single("Results single line", results_single_line, 35);
    check_single("Results continue text", results_continue, 35);

    // Check if results title matches the line widths
    println!("\n=== RESULTS WIDTH COMPARISON ===");
    let double_w = UnicodeWidthStr::width(results_double_line);
    let title_w = UnicodeWidthStr::width(results_title);
    let single_w = UnicodeWidthStr::width(results_single_line);
    let continue_w = UnicodeWidthStr::width(results_continue);
    println!("  Double line: {}", double_w);
    println!("  Title:       {}", title_w);
    println!("  Single line: {}", single_w);
    println!("  Continue:    {}", continue_w);
    if double_w == title_w && title_w == single_w && single_w == continue_w {
        println!("  ✓ All results screen elements have same width");
    } else {
        println!("  ✗ RESULTS SCREEN ELEMENTS HAVE INCONSISTENT WIDTHS");
    }

    // ========== EQUALIZER CHANNEL LABELS ==========
    println!("\n=== EQUALIZER CHANNEL LABELS ===");
    // Each channel label should be same width: "  CHAN N  " = 10 chars
    check_single("CHAN 1 label", "  CHAN 1  ", 10);
    check_single("CHAN 2 label", "  CHAN 2  ", 10);
    check_single("CHAN 3 label", "  CHAN 3  ", 10);
    check_single("CHAN 4 label", "  CHAN 4  ", 10);

    // ========== MOD TRACKER TITLE ==========
    println!("\n=== MOD TRACKER TITLE ===");
    let tracker_title = " ◆ M O D   T R A C K E R ◆ ";
    let tracker_prefix = "▄▀";
    let tracker_suffix = "▀▄";
    check_single("Tracker title", tracker_title, 27);
    check_single("Tracker prefix", tracker_prefix, 2);
    check_single("Tracker suffix", tracker_suffix, 2);

    println!("\n======== CHECK COMPLETE ========\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_consistency() {
        run_checks();
    }

    #[test]
    fn test_border_positions() {
        // Test at multiple sizes to catch width-dependent bugs
        for (width, height) in [(80, 30), (120, 40), (204, 58)] {
            test_borders_at_size(width, height);
        }
    }

    fn test_borders_at_size(width: u16, height: u16) {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;
        use crate::chart::Chart;
        use crate::game::Game;
        use crate::render::render;

        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();

        // Create a minimal game state
        let chart = Chart::generic("Test Song", 120.0);
        let game = Game::new(chart);

        // Render the game
        terminal.draw(|f| render(f, &game)).unwrap();

        // Get the buffer and check border positions
        let buffer = terminal.backend().buffer();
        let width = buffer.area.width as usize;
        let height = buffer.area.height as usize;

        println!("\n======== BORDER POSITION CHECK ========");
        println!("Terminal size: {}x{}", width, height);

        // Define expected border characters
        let left_borders = ['│', '┌', '├', '└'];
        let right_borders = ['│', '┐', '┤', '┘'];

        let mut errors = Vec::new();

        for y in 0..height {
            let first_char = buffer.get(0, y as u16).symbol().chars().next().unwrap_or(' ');
            let last_char = buffer.get((width - 1) as u16, y as u16).symbol().chars().next().unwrap_or(' ');

            let first_ok = left_borders.contains(&first_char);
            let last_ok = right_borders.contains(&last_char);

            let status = if first_ok && last_ok {
                "✓"
            } else {
                "✗"
            };

            println!(
                "{} Row {:2}: first='{}' ({}), last='{}' ({})",
                status,
                y,
                first_char,
                if first_ok { "ok" } else { "BAD" },
                last_char,
                if last_ok { "ok" } else { "BAD" }
            );

            if !first_ok {
                errors.push(format!("Row {}: first char '{}' is not a left border", y, first_char));
            }
            if !last_ok {
                errors.push(format!("Row {}: last char '{}' is not a right border", y, last_char));
            }
        }

        println!("\n======== BORDER CHECK COMPLETE ========\n");

        // Also print a few rows to visually inspect
        if width >= 200 {
            println!("=== VISUAL DUMP OF ROWS ===");
            println!("Showing: cols 0-15, cols {}-{} (center), cols {}-{}",
                     width/2 - 30, width/2 + 30, width - 16, width - 1);
            for y in 0..height.min(25) {
                let mut first = String::new();
                let mut center = String::new();
                let mut last = String::new();
                // First 16 chars
                for x in 0..16.min(width) {
                    first.push_str(buffer.get(x as u16, y as u16).symbol());
                }
                // Center 60 chars
                let center_start = (width / 2).saturating_sub(30);
                let center_end = (width / 2 + 30).min(width);
                for x in center_start..center_end {
                    center.push_str(buffer.get(x as u16, y as u16).symbol());
                }
                // Last 16 chars
                for x in (width - 16).max(0)..width {
                    last.push_str(buffer.get(x as u16, y as u16).symbol());
                }
                println!("Row {:2}: [{}]...[{}]...[{}]", y, first, center, last);
            }
        }

        if !errors.is_empty() {
            println!("ERRORS FOUND:");
            for err in &errors {
                println!("  - {}", err);
            }
            panic!("Border position errors found at {}x{}: {}", width, height, errors.len());
        }
    }
}

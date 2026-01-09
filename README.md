# Rhythm MOD

A terminal-based taiko drum rhythm game that plays classic Amiga MOD tracker music.

![Rhythm MOD Screenshot](screenshot.png)

## Features

- Play along to MOD tracker music files (4-channel ProTracker format)
- Taiko-style gameplay with DON (center) and KA (rim) notes
- Retro 4-channel VU meter equalizer visualization
- Neon arcade aesthetic with bold colors
- Score tracking with combo system
- Song selection menu

## Screenshot

```
┌──────────────────────────────────────────────────────────────────────────┐
│ ▄▀▀▀▄  ◆ axelf ◆                                            00012500 │
│ SOUL ▐████████████░░░░░░░░▌  67%                                       │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│    ▄▄███▄▄    ┃░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░│
│   █▓▓▓▓▓▓▓█   ┃─────────────────────────────▄███▄─────▄▀▀▀▄────────────│
│  █▓▓█████▓▓█  ┃═════════════════════════════█████═════█ ◆ █════════════│
│  █▓██   ██▓█  ┃─────────────────────────────▀███▀─────▀▄▄▄▀────────────│
│  █▓▓█████▓▓█  ┃░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░│
│   █▓▓▓▓▓▓▓█   ┃                                                        │
│    ▀▀███▀▀    ┃                                                        │
├──────────────────────────────────────────────────────────────────────────┤
│                     ▄▀ ◆ M O D   T R A C K E R ◆ ▀▄                     │
│         ▐██████████▌ ▐██████████▌ ▐██████████▌ ▐██████████▌             │
│         ▐██████████▌ ▐██████████▌ ▐          ▌ ▐██████████▌             │
│         ▐██████████▌ ▐──────────▌ ▐          ▌ ▐██████████▌             │
│         ▐██████████▌ ▐          ▌ ▐          ▌ ▐          ▌             │
│         ▐▄▄▄▄▄▄▄▄▄▄▌ ▐          ▌ ▐          ▌ ▐          ▌             │
│         ▐          ▌ ▐          ▌ ▐          ▌ ▐          ▌             │
│         ▐          ▌ ▐          ▌ ▐          ▌ ▐          ▌             │
│         ▌  CHAN 1  ▐ ▌  CHAN 2  ▐ ▌  CHAN 3  ▐ ▌  CHAN 4  ▐             │
├──────────────────────────────────────────────────────────────────────────┤
│                        ★ ★ ★  P E R F E C T  ★ ★ ★                      │
│                            15  C O M B O                                │
├──────────────────────────────────────────────────────────────────────────┤
│   ███ DON [D][F]        █◆█ KA [J][K]        [ESC] quit                 │
└──────────────────────────────────────────────────────────────────────────┘
```

## Controls

| Key | Action |
|-----|--------|
| `D` / `F` | DON - Center drum hit (red notes) |
| `J` / `K` | KA - Rim hit (blue notes) |
| `↑` / `↓` | Navigate song menu |
| `Enter` | Select song |
| `Esc` / `Q` | Quit |

## Timing Windows

- **Perfect**: ±50ms
- **Good**: ±100ms
- **Miss**: >150ms

## Installation

```bash
# Clone the repository
git clone https://github.com/elichen/rhythm_mod.git
cd rhythm_mod

# Build and run
cargo run --release
```

## Adding Songs

Place `.mod` files (4-channel ProTracker format) in the `assets/` directory. The game will automatically detect and list them in the song selection menu.

You can find MOD files at:
- [The Mod Archive](https://modarchive.org/)
- [Amiga Music Preservation](https://amp.dascene.net/)

**Note**: Only classic 4-channel ProTracker MOD files are supported. S3M, XM, and IT formats are not compatible.

## Dependencies

- [ratatui](https://crates.io/crates/ratatui) - Terminal UI framework
- [crossterm](https://crates.io/crates/crossterm) - Terminal input handling
- [rodio](https://crates.io/crates/rodio) - Audio playback
- [mod_player](https://crates.io/crates/mod_player) - MOD file playback

## License

MIT

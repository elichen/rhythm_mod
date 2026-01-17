//! Minimal MOD file parser for extracting beat timing information.
//!
//! Parses the binary MOD format to extract pattern data and convert
//! note triggers to chart notes with proper timing.

use std::fs;
use std::path::Path;

/// A note event extracted from the MOD file
#[derive(Debug, Clone)]
pub struct BeatEvent {
    /// Time in milliseconds from song start
    pub time_ms: u64,
    /// Sample number (1-31) that triggered
    pub sample: u8,
    /// Period value (pitch) - lower = higher pitch
    pub period: u16,
}

/// Parsed MOD file data
pub struct ModData {
    pub song_length: u8,
    pub pattern_table: [u8; 128],
    pub patterns: Vec<Pattern>,
    /// Sample names (up to 31)
    pub sample_names: Vec<String>,
}

impl ModData {
    /// Check if a sample is a "Don" sound (bass/kick/tom) based on its name
    pub fn is_don_sample(&self, sample_num: u8) -> bool {
        if sample_num == 0 || sample_num as usize > self.sample_names.len() {
            return false;
        }
        let name = &self.sample_names[sample_num as usize - 1];
        let lower = name.to_lowercase();
        // Bass drum and tom → Don (center hit)
        // Snare is Ka (rim hit), not Don
        lower.contains("bass") || lower.contains("kick") || lower.contains("tom")
    }
}

/// A pattern with 64 rows of notes
pub struct Pattern {
    pub rows: Vec<[NoteData; 4]>,
}

/// A single note in a pattern
#[derive(Default, Clone, Copy)]
pub struct NoteData {
    pub sample: u8,
    pub period: u16,
    pub effect: u8,
    pub effect_param: u8,
}

impl NoteData {
    /// Check if this is a SetSpeed effect (Fxx where x < 0x20)
    pub fn get_speed_change(&self) -> Option<u8> {
        if self.effect == 0xF && self.effect_param > 0 && self.effect_param < 0x20 {
            Some(self.effect_param)
        } else {
            None
        }
    }

    /// Check if this is a SetTempo effect (Fxx where x >= 0x20)
    pub fn get_tempo_change(&self) -> Option<u8> {
        if self.effect == 0xF && self.effect_param >= 0x20 {
            Some(self.effect_param)
        } else {
            None
        }
    }

    /// Check if this is a Pattern Delay effect (EEx)
    pub fn get_pattern_delay(&self) -> Option<u8> {
        if self.effect == 0xE && (self.effect_param >> 4) == 0xE {
            let delay = self.effect_param & 0x0F;
            if delay > 0 { Some(delay) } else { None }
        } else {
            None
        }
    }
}

const DEFAULT_SPEED: u8 = 6;
const DEFAULT_BPM: u16 = 125;

fn bpm_to_tick_ms(bpm: u16) -> f64 {
    2500.0 / bpm as f64
}

/// Parse a MOD file and extract beat events
pub fn parse_mod_file(path: &Path) -> anyhow::Result<ModData> {
    let data = fs::read(path)?;

    if data.len() < 600 {
        anyhow::bail!("File too small to be a valid MOD file");
    }

    // Detect format: check for format identifier at offset 1080
    // If present, it's a 31-sample MOD; otherwise 15-sample
    let (num_samples, num_channels, song_length_offset, pattern_table_offset, pattern_start) =
        if data.len() >= 1084 {
            let format_id = &data[1080..1084];
            match format_id {
                b"M.K." | b"M!K!" | b"4CHN" | b"FLT4" => (31, 4, 950, 952, 1084),
                b"6CHN" => (31, 6, 950, 952, 1084),
                b"8CHN" | b"FLT8" => (31, 8, 950, 952, 1084),
                _ => {
                    // No valid format ID - likely 15-sample format
                    // 15-sample: song_length at 470, pattern_table at 472, patterns at 600
                    (15, 4, 470, 472, 600)
                }
            }
        } else {
            // File too small for 31-sample, assume 15-sample
            (15, 4, 470, 472, 600)
        };

    // Parse sample names (each sample record is 30 bytes, name is first 22 bytes)
    let mut sample_names = Vec::with_capacity(num_samples);
    for i in 0..num_samples {
        let name_offset = 20 + i * 30;
        if name_offset + 22 <= data.len() {
            let name_bytes = &data[name_offset..name_offset + 22];
            let name = name_bytes
                .iter()
                .take_while(|&&b| b != 0)
                .map(|&b| b as char)
                .collect::<String>()
                .trim()
                .to_string();
            sample_names.push(name);
        } else {
            sample_names.push(String::new());
        }
    }

    // Song length
    let song_length = data[song_length_offset];

    // Pattern table (128 bytes)
    let mut pattern_table = [0u8; 128];
    let table_end = pattern_table_offset + 128;
    if table_end <= data.len() {
        pattern_table.copy_from_slice(&data[pattern_table_offset..table_end]);
    }

    // Find highest pattern number to know how many patterns exist
    // Only look at first song_length entries
    let max_pattern = pattern_table[..song_length as usize]
        .iter()
        .copied()
        .max()
        .unwrap_or(0);

    // Each pattern = 64 rows × num_channels × 4 bytes per note
    let bytes_per_pattern = 64 * num_channels * 4;

    let mut patterns = Vec::new();
    for pat_idx in 0..=max_pattern {
        let pat_offset = pattern_start + (pat_idx as usize) * bytes_per_pattern;

        if pat_offset + bytes_per_pattern > data.len() {
            // Not enough data, create empty pattern
            patterns.push(Pattern {
                rows: vec![[NoteData::default(); 4]; 64],
            });
            continue;
        }

        let mut rows = Vec::with_capacity(64);
        for row in 0..64 {
            let row_offset = pat_offset + row * num_channels * 4;
            let mut notes = [NoteData::default(); 4];

            for ch in 0..num_channels.min(4) {
                let note_offset = row_offset + ch * 4;
                if note_offset + 4 <= data.len() {
                    notes[ch] = parse_note(&data[note_offset..note_offset + 4]);
                }
            }
            rows.push(notes);
        }
        patterns.push(Pattern { rows });
    }

    Ok(ModData {
        song_length,
        pattern_table,
        patterns,
        sample_names,
    })
}

/// Parse 4 bytes into a NoteData
fn parse_note(bytes: &[u8]) -> NoteData {
    // MOD note format:
    // Byte 0: Upper 4 bits of sample, lower 4 bits are upper 4 bits of period
    // Byte 1: Lower 8 bits of period
    // Byte 2: Lower 4 bits of sample (upper), effect (lower)
    // Byte 3: Effect parameter

    let sample_hi = bytes[0] & 0xF0;
    let sample_lo = (bytes[2] & 0xF0) >> 4;
    let sample = sample_hi | sample_lo;

    let period_hi = (bytes[0] & 0x0F) as u16;
    let period_lo = bytes[1] as u16;
    let period = (period_hi << 8) | period_lo;

    let effect = bytes[2] & 0x0F;
    let effect_param = bytes[3];

    NoteData {
        sample,
        period,
        effect,
        effect_param,
    }
}

/// Extract beat events from parsed MOD data
pub fn extract_beats(mod_data: &ModData) -> Vec<BeatEvent> {
    let mut events = Vec::new();
    let mut current_speed: u32 = DEFAULT_SPEED as u32;
    let mut current_bpm: u16 = DEFAULT_BPM;
    let mut tick_ms = bpm_to_tick_ms(current_bpm);
    // mod_player waits an extra tick before the first line is played.
    let mut time_ms: f64 = tick_ms * (current_speed as f64 + 1.0);

    for pat_pos in 0..mod_data.song_length as usize {
        let pat_idx = mod_data.pattern_table[pat_pos] as usize;

        if pat_idx >= mod_data.patterns.len() {
            let row_ticks = current_speed.max(1) as f64;
            time_ms += 64.0 * row_ticks * tick_ms;
            continue;
        }

        let pattern = &mod_data.patterns[pat_idx];

        for row in pattern.rows.iter() {
            let row_time_ms = time_ms.round() as u64;

            // Check each channel for note triggers and speed changes
            for note in row.iter() {
                if note.sample != 0 && note.period != 0 {
                    events.push(BeatEvent {
                        time_ms: row_time_ms,
                        sample: note.sample,
                        period: note.period,
                    });
                }
            }

            let mut delay_ticks: u32 = 0;

            for note in row.iter() {
                if let Some(new_speed) = note.get_speed_change() {
                    current_speed = new_speed as u32;
                }
                if let Some(new_bpm) = note.get_tempo_change() {
                    current_bpm = new_bpm as u16;
                    tick_ms = bpm_to_tick_ms(current_bpm);
                }
                if let Some(delay) = note.get_pattern_delay() {
                    delay_ticks = delay as u32;
                }
            }

            let row_ticks = current_speed.max(1) + delay_ticks;
            time_ms += (row_ticks as f64) * tick_ms;
        }
    }

    events
}

/// Filter beat events to avoid too-dense notes and convert to rhythm game timing
pub fn filter_beats_for_gameplay(events: Vec<BeatEvent>, min_gap_ms: u64) -> Vec<BeatEvent> {
    if events.is_empty() {
        return events;
    }

    let mut filtered = Vec::new();
    let mut last_time: Option<u64> = None;

    for event in events {
        match last_time {
            Some(lt) if event.time_ms < lt + min_gap_ms => {
                // Skip - too close to previous note
                continue;
            }
            _ => {
                last_time = Some(event.time_ms);
                filtered.push(event);
            }
        }
    }

    filtered
}

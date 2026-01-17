use std::path::Path;
use crate::mod_parser::{parse_mod_file, extract_beats, filter_beats_for_gameplay};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NoteType {
    DonLeft,  // Center hit left (red) - D key
    DonRight, // Center hit right (red) - F key
    KaLeft,   // Rim hit left (blue) - J key
    KaRight,  // Rim hit right (blue) - K key
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Difficulty {
    Easy,
    #[default]
    Medium,
    Hard,
}

impl Difficulty {
    /// Minimum gap between notes in milliseconds
    pub fn min_gap_ms(&self) -> u64 {
        match self {
            Difficulty::Easy => 300,   // Fewer notes
            Difficulty::Medium => 150, // Moderate
            Difficulty::Hard => 50,    // Many notes
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Difficulty::Easy => "Easy",
            Difficulty::Medium => "Medium",
            Difficulty::Hard => "Hard",
        }
    }
}

#[derive(Clone)]
pub struct Note {
    pub note_type: NoteType,
    pub time_ms: u64,
    pub hit: bool, // Whether this note has been processed
}

/// Raw beat event for EQ visualization (includes sample info)
#[derive(Clone)]
pub struct EqBeat {
    pub time_ms: u64,
    pub sample: u8,
}

pub struct Chart {
    pub title: String,
    #[allow(dead_code)]
    pub bpm: f32,
    pub notes: Vec<Note>,
    /// All beat events for EQ visualization (unfiltered, includes sample numbers)
    pub eq_beats: Vec<EqBeat>,
}

impl Chart {
    /// Creates a generic chart for any song at a given BPM
    pub fn generic(title: &str, bpm: f32) -> Self {
        let beat_ms = (60_000.0 / bpm) as u64;
        let half_beat = beat_ms / 2;

        let mut notes = Vec::new();
        let start_offset = 2000; // 2 second lead-in

        // Create a varied pattern that works with most songs
        // 16 bars of rhythmic variety
        let mut time = start_offset;

        // Section 1: Basic groove (4 bars) - alternating L/R
        for _ in 0..4 {
            notes.push(Note { note_type: NoteType::DonLeft, time_ms: time, hit: false });
            notes.push(Note { note_type: NoteType::KaRight, time_ms: time + half_beat, hit: false });
            notes.push(Note { note_type: NoteType::DonRight, time_ms: time + beat_ms, hit: false });
            notes.push(Note { note_type: NoteType::KaLeft, time_ms: time + beat_ms + half_beat, hit: false });
            notes.push(Note { note_type: NoteType::DonLeft, time_ms: time + beat_ms * 2, hit: false });
            notes.push(Note { note_type: NoteType::KaRight, time_ms: time + beat_ms * 2 + half_beat, hit: false });
            notes.push(Note { note_type: NoteType::DonRight, time_ms: time + beat_ms * 3, hit: false });
            time += beat_ms * 4;
        }

        // Section 2: More active (4 bars)
        for _ in 0..4 {
            notes.push(Note { note_type: NoteType::DonLeft, time_ms: time, hit: false });
            notes.push(Note { note_type: NoteType::DonRight, time_ms: time + half_beat, hit: false });
            notes.push(Note { note_type: NoteType::KaLeft, time_ms: time + beat_ms, hit: false });
            notes.push(Note { note_type: NoteType::KaRight, time_ms: time + beat_ms + half_beat, hit: false });
            notes.push(Note { note_type: NoteType::DonLeft, time_ms: time + beat_ms * 2, hit: false });
            notes.push(Note { note_type: NoteType::KaLeft, time_ms: time + beat_ms * 2 + half_beat, hit: false });
            notes.push(Note { note_type: NoteType::DonRight, time_ms: time + beat_ms * 3, hit: false });
            notes.push(Note { note_type: NoteType::KaRight, time_ms: time + beat_ms * 3 + half_beat, hit: false });
            time += beat_ms * 4;
        }

        // Section 3: Syncopated (4 bars)
        for _ in 0..4 {
            notes.push(Note { note_type: NoteType::DonLeft, time_ms: time, hit: false });
            notes.push(Note { note_type: NoteType::KaRight, time_ms: time + beat_ms / 4, hit: false });
            notes.push(Note { note_type: NoteType::DonRight, time_ms: time + beat_ms, hit: false });
            notes.push(Note { note_type: NoteType::DonLeft, time_ms: time + beat_ms * 2, hit: false });
            notes.push(Note { note_type: NoteType::KaLeft, time_ms: time + beat_ms * 2 + beat_ms / 4, hit: false });
            notes.push(Note { note_type: NoteType::DonRight, time_ms: time + beat_ms * 3, hit: false });
            notes.push(Note { note_type: NoteType::KaRight, time_ms: time + beat_ms * 3 + half_beat, hit: false });
            time += beat_ms * 4;
        }

        // Section 4: Finale (4 bars)
        let types = [NoteType::DonLeft, NoteType::KaRight, NoteType::DonRight, NoteType::KaLeft];
        for bar in 0..4 {
            for beat in 0..4 {
                let beat_start = time + beat * beat_ms;
                notes.push(Note {
                    note_type: types[(beat as usize + bar as usize) % 4],
                    time_ms: beat_start,
                    hit: false,
                });
                if bar >= 2 {
                    notes.push(Note {
                        note_type: types[(beat as usize + bar as usize + 2) % 4],
                        time_ms: beat_start + half_beat,
                        hit: false,
                    });
                }
            }
            time += beat_ms * 4;
        }

        Chart {
            title: title.to_string(),
            bpm,
            notes,
            eq_beats: Vec::new(), // No EQ data for generic charts
        }
    }

    /// Creates a chart synchronized to a MOD file's beat pattern
    pub fn from_mod_file(path: &Path, title: &str, difficulty: Difficulty) -> anyhow::Result<Self> {
        let mod_data = parse_mod_file(path)?;
        let beats = extract_beats(&mod_data);

        // Store all beats for EQ visualization (synced to audio timing)
        let eq_beats: Vec<EqBeat> = beats
            .iter()
            .map(|b| EqBeat {
                time_ms: b.time_ms,
                sample: b.sample,
            })
            .collect();

        // Filter based on difficulty (larger gap = fewer notes)
        let filtered = filter_beats_for_gameplay(beats, difficulty.min_gap_ms());

        // Calculate median period for pitch-based L/R split
        let median_period = {
            let mut periods: Vec<u16> = filtered.iter().map(|b| b.period).collect();
            periods.sort();
            if periods.is_empty() {
                428 // Default to middle C period if no notes
            } else {
                periods[periods.len() / 2]
            }
        };

        // Map notes based on instrument type (Don/Ka) and pitch (Left/Right)
        // Don = bass/kick/tom, Ka = everything else
        // Left = lower pitch (higher period), Right = higher pitch (lower period)
        let notes: Vec<Note> = filtered
            .into_iter()
            .map(|beat| {
                let is_don = mod_data.is_don_sample(beat.sample);
                let is_low_pitch = beat.period >= median_period;

                let note_type = match (is_don, is_low_pitch) {
                    (true, true) => NoteType::DonLeft,   // D - bass, low pitch
                    (true, false) => NoteType::DonRight, // F - bass, high pitch
                    (false, true) => NoteType::KaLeft,   // J - other, low pitch
                    (false, false) => NoteType::KaRight, // K - other, high pitch
                };

                Note {
                    note_type,
                    time_ms: beat.time_ms,
                    hit: false,
                }
            })
            .collect();

        // Estimate BPM from note timing (rough approximation)
        let bpm = if notes.len() >= 2 {
            let total_time = notes.last().unwrap().time_ms - notes.first().unwrap().time_ms;
            let beats_count = notes.len() as f32;
            if total_time > 0 {
                (beats_count * 60_000.0) / total_time as f32
            } else {
                120.0
            }
        } else {
            120.0
        };

        Ok(Chart {
            title: title.to_string(),
            bpm,
            notes,
            eq_beats,
        })
    }
}

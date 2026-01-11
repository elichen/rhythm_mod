use std::path::Path;
use crate::mod_parser::{parse_mod_file, extract_beats, filter_beats_for_gameplay};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NoteType {
    Don, // Center hit (red) - D/F keys
    Ka,  // Rim hit (blue) - J/K keys
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

        // Section 1: Basic groove (4 bars)
        for _ in 0..4 {
            notes.push(Note { note_type: NoteType::Don, time_ms: time, hit: false });
            notes.push(Note { note_type: NoteType::Ka, time_ms: time + half_beat, hit: false });
            notes.push(Note { note_type: NoteType::Don, time_ms: time + beat_ms, hit: false });
            notes.push(Note { note_type: NoteType::Ka, time_ms: time + beat_ms + half_beat, hit: false });
            notes.push(Note { note_type: NoteType::Don, time_ms: time + beat_ms * 2, hit: false });
            notes.push(Note { note_type: NoteType::Ka, time_ms: time + beat_ms * 2 + half_beat, hit: false });
            notes.push(Note { note_type: NoteType::Don, time_ms: time + beat_ms * 3, hit: false });
            time += beat_ms * 4;
        }

        // Section 2: More active (4 bars)
        for _ in 0..4 {
            notes.push(Note { note_type: NoteType::Don, time_ms: time, hit: false });
            notes.push(Note { note_type: NoteType::Don, time_ms: time + half_beat, hit: false });
            notes.push(Note { note_type: NoteType::Ka, time_ms: time + beat_ms, hit: false });
            notes.push(Note { note_type: NoteType::Ka, time_ms: time + beat_ms + half_beat, hit: false });
            notes.push(Note { note_type: NoteType::Don, time_ms: time + beat_ms * 2, hit: false });
            notes.push(Note { note_type: NoteType::Ka, time_ms: time + beat_ms * 2 + half_beat, hit: false });
            notes.push(Note { note_type: NoteType::Don, time_ms: time + beat_ms * 3, hit: false });
            notes.push(Note { note_type: NoteType::Ka, time_ms: time + beat_ms * 3 + half_beat, hit: false });
            time += beat_ms * 4;
        }

        // Section 3: Syncopated (4 bars)
        for _ in 0..4 {
            notes.push(Note { note_type: NoteType::Don, time_ms: time, hit: false });
            notes.push(Note { note_type: NoteType::Ka, time_ms: time + beat_ms / 4, hit: false });
            notes.push(Note { note_type: NoteType::Don, time_ms: time + beat_ms, hit: false });
            notes.push(Note { note_type: NoteType::Don, time_ms: time + beat_ms * 2, hit: false });
            notes.push(Note { note_type: NoteType::Ka, time_ms: time + beat_ms * 2 + beat_ms / 4, hit: false });
            notes.push(Note { note_type: NoteType::Don, time_ms: time + beat_ms * 3, hit: false });
            notes.push(Note { note_type: NoteType::Ka, time_ms: time + beat_ms * 3 + half_beat, hit: false });
            time += beat_ms * 4;
        }

        // Section 4: Finale (4 bars)
        for bar in 0..4 {
            // Increasing intensity
            for beat in 0..4 {
                let beat_start = time + beat * beat_ms;
                notes.push(Note {
                    note_type: if (beat + bar) % 2 == 0 { NoteType::Don } else { NoteType::Ka },
                    time_ms: beat_start,
                    hit: false,
                });
                if bar >= 2 {
                    // Add off-beats in last 2 bars
                    notes.push(Note {
                        note_type: NoteType::Ka,
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
    pub fn from_mod_file(path: &Path, title: &str) -> anyhow::Result<Self> {
        let mod_data = parse_mod_file(path)?;
        let beats = extract_beats(&mod_data);

        // Store all beats for EQ visualization (with lead-in offset)
        let lead_in_ms: u64 = 2000;
        let eq_beats: Vec<EqBeat> = beats
            .iter()
            .map(|b| EqBeat {
                time_ms: b.time_ms + lead_in_ms,
                sample: b.sample,
            })
            .collect();

        // Filter to avoid notes that are too close together (min 100ms gap)
        let filtered = filter_beats_for_gameplay(beats, 100);

        let notes: Vec<Note> = filtered
            .into_iter()
            .map(|beat| {
                // Map samples to note types:
                // Low sample numbers (1-4) are typically bass/kick drums -> Don
                // Higher sample numbers are typically snare/hi-hat -> Ka
                let note_type = if beat.sample <= 4 {
                    NoteType::Don
                } else {
                    NoteType::Ka
                };

                Note {
                    note_type,
                    time_ms: beat.time_ms + lead_in_ms,
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

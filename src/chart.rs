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

pub struct Chart {
    pub title: String,
    #[allow(dead_code)]
    pub bpm: f32,
    pub notes: Vec<Note>,
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
        }
    }
}

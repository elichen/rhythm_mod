use std::time::Instant;

use crate::chart::{Chart, NoteType};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HitResult {
    Perfect,
    Good,
    Miss,
    Wrong, // Wrong note type
}

pub struct Game {
    pub chart: Chart,
    pub start_time: Instant,
    pub score: u32,
    pub combo: u32,
    pub max_combo: u32,
    pub last_hit_result: Option<HitResult>,
    pub result_display_until: u64, // Show result until this time
    pub perfect_count: u32,
    pub good_count: u32,
    pub miss_count: u32,
    // Equalizer channel levels (0.0 - 1.0) for retro visualizer
    pub eq_levels: [f32; 4],
    pub eq_targets: [f32; 4],
    pub last_hit_time: u64,
    // For real EQ: track position in beat list
    eq_beat_index: usize,
    // Recent sample triggers for each EQ channel (decay over time)
    eq_channel_energy: [f32; 4],
}

// Timing windows in milliseconds
const PERFECT_WINDOW: i64 = 50;
const GOOD_WINDOW: i64 = 100;
const MISS_WINDOW: i64 = 150; // Notes are auto-missed after this

impl Game {
    pub fn new(chart: Chart) -> Self {
        Game {
            chart,
            start_time: Instant::now(),
            score: 0,
            combo: 0,
            max_combo: 0,
            last_hit_result: None,
            result_display_until: 0,
            perfect_count: 0,
            good_count: 0,
            miss_count: 0,
            eq_levels: [0.0; 4],
            eq_targets: [0.0; 4],
            last_hit_time: 0,
            eq_beat_index: 0,
            eq_channel_energy: [0.0; 4],
        }
    }

    /// Map a sample number to an EQ channel (0-3)
    /// Low samples = bass (left), high samples = treble (right)
    fn sample_to_channel(sample: u8) -> usize {
        match sample {
            1..=3 => 0,   // Bass/kick -> channel 0
            4..=6 => 1,   // Low-mid -> channel 1
            7..=10 => 2,  // High-mid -> channel 2
            _ => 3,       // Hi-hat/high -> channel 3
        }
    }

    /// Update equalizer based on real MOD playback
    pub fn update_equalizer(&mut self) {
        let time = self.current_time_ms();

        // Process any beats that have occurred since last update
        while self.eq_beat_index < self.chart.eq_beats.len() {
            let beat = &self.chart.eq_beats[self.eq_beat_index];
            if beat.time_ms <= time {
                // This beat just happened - boost the appropriate channel
                let channel = Self::sample_to_channel(beat.sample);
                self.eq_channel_energy[channel] = 1.0;
                self.eq_beat_index += 1;
            } else {
                break;
            }
        }

        // Update each channel
        for i in 0..4 {
            // Decay the channel energy
            self.eq_channel_energy[i] *= 0.85; // Fast decay

            // Set target based on current energy
            self.eq_targets[i] = self.eq_channel_energy[i];

            // Smooth interpolation toward target
            let diff = self.eq_targets[i] - self.eq_levels[i];
            if diff.abs() > 0.005 {
                // Rise FAST, fall slower (punchy VU meter feel)
                let speed = if diff > 0.0 { 0.6 } else { 0.15 };
                self.eq_levels[i] += diff * speed;
            }
        }
    }

    /// Start the game timer (call right when audio starts)
    pub fn start(&mut self) {
        self.start_time = Instant::now();
    }

    /// Get current game time in milliseconds
    pub fn current_time_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }

    /// Process a hit input from the player
    pub fn process_hit(&mut self, hit_type: NoteType) -> HitResult {
        let current_time = self.current_time_ms() as i64;

        // Find the closest unhit note within the hit window
        let mut best_match: Option<(usize, i64)> = None;

        for (i, note) in self.chart.notes.iter().enumerate() {
            if note.hit {
                continue;
            }

            let diff = (note.time_ms as i64 - current_time).abs();

            // Only consider notes within the good window
            if diff <= GOOD_WINDOW {
                if best_match.is_none() || diff < best_match.unwrap().1 {
                    best_match = Some((i, diff));
                }
            }
        }

        let result = match best_match {
            Some((idx, diff)) => {
                let note = &self.chart.notes[idx];

                // Check if correct note type
                if note.note_type != hit_type {
                    self.break_combo();
                    HitResult::Wrong
                } else {
                    // Mark note as hit
                    self.chart.notes[idx].hit = true;

                    if diff <= PERFECT_WINDOW {
                        self.add_score(300);
                        self.combo += 1;
                        self.perfect_count += 1;
                        HitResult::Perfect
                    } else {
                        self.add_score(100);
                        self.combo += 1;
                        self.good_count += 1;
                        HitResult::Good
                    }
                }
            }
            None => {
                // No note nearby - it's a miss
                self.break_combo();
                HitResult::Miss
            }
        };

        // Update max combo
        if self.combo > self.max_combo {
            self.max_combo = self.combo;
        }

        // Show result for 500ms
        self.last_hit_result = Some(result);
        self.result_display_until = self.current_time_ms() + 500;

        // Pulse equalizer on hit
        if result == HitResult::Perfect || result == HitResult::Good {
            self.last_hit_time = self.current_time_ms();
        }

        result
    }

    /// Check for notes that should be auto-missed (passed without hitting)
    pub fn check_missed_notes(&mut self) {
        let current_time = self.current_time_ms() as i64;

        // Collect indices of missed notes first to avoid borrow issues
        let missed_indices: Vec<usize> = self
            .chart
            .notes
            .iter()
            .enumerate()
            .filter(|(_, note)| {
                !note.hit && current_time - note.time_ms as i64 > MISS_WINDOW
            })
            .map(|(i, _)| i)
            .collect();

        // Now apply the changes
        for idx in missed_indices {
            self.chart.notes[idx].hit = true;
            self.miss_count += 1;
            self.combo = 0;
            self.last_hit_result = Some(HitResult::Miss);
            self.result_display_until = self.current_time_ms() + 500;
        }
    }

    /// Check if the song is complete
    pub fn is_complete(&self) -> bool {
        let current_time = self.current_time_ms();
        if let Some(last_note) = self.chart.notes.last() {
            // Song is complete 2 seconds after last note
            current_time > last_note.time_ms + 2000
        } else {
            true
        }
    }

    fn add_score(&mut self, base: u32) {
        // Combo bonus
        let bonus = (self.combo / 10) * 10;
        self.score += base + bonus;
    }

    fn break_combo(&mut self) {
        self.combo = 0;
    }

    /// Clear hit result display if expired
    pub fn update_result_display(&mut self) {
        if self.current_time_ms() > self.result_display_until {
            self.last_hit_result = None;
        }
    }
}

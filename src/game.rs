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
        }
    }

    /// Update equalizer animation (call each frame)
    pub fn update_equalizer(&mut self) {
        let time = self.current_time_ms();

        // Generate pseudo-random targets based on time for ambient animation
        // Each channel updates at different rates for organic movement
        let rates = [67, 83, 53, 97]; // Prime numbers for varied timing

        for i in 0..4 {
            // Update target periodically
            if time % rates[i] < 20 {
                // Multiple sine waves for more organic movement
                let seed1 = (time as f32 / rates[i] as f32) + (i as f32 * 1.7);
                let seed2 = (time as f32 / (rates[i] as f32 * 0.7)) + (i as f32 * 2.3);
                let wave1 = (seed1 * 0.15).sin() * 0.5 + 0.5;
                let wave2 = (seed2 * 0.23).sin() * 0.3 + 0.5;
                let base = (wave1 * 0.6 + wave2 * 0.4) * 0.5; // Stronger base animation

                // BIG boost when hitting notes
                let time_since_hit = time.saturating_sub(self.last_hit_time);
                let hit_boost = if time_since_hit < 150 {
                    // Sharp attack, quick decay
                    let t = time_since_hit as f32 / 150.0;
                    0.7 * (1.0 - t * t) // Quadratic decay
                } else if time_since_hit < 400 {
                    0.2 // Sustain
                } else {
                    0.0
                };

                // Combo adds sustained energy - more dramatic
                let combo_boost = (self.combo as f32 / 30.0).min(0.4);

                // Randomize which channel gets the most energy
                let channel_variance = ((seed1 * 3.0).sin() * 0.15).abs();

                self.eq_targets[i] = (base + hit_boost + combo_boost + channel_variance).min(1.0);
            }

            // Smooth interpolation toward target
            let diff = self.eq_targets[i] - self.eq_levels[i];
            if diff.abs() > 0.005 {
                // Rise FAST, fall slow (punchy VU meter feel)
                let speed = if diff > 0.0 { 0.45 } else { 0.06 };
                self.eq_levels[i] += diff * speed;
            }
        }
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

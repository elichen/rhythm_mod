use mod_player::{read_mod_file, next_sample, PlayerState, Song};
use rodio::source::Source;
use rodio::{OutputStream, OutputStreamHandle, Sink};
use std::time::Duration;
use std::path::Path;

use crate::chart::NoteType;

const SAMPLE_RATE: u32 = 48000;

pub struct Audio {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    music_sink: Option<Sink>,
}

impl Audio {
    pub fn new() -> anyhow::Result<Self> {
        let (stream, stream_handle) = OutputStream::try_default()?;
        Ok(Audio {
            _stream: stream,
            stream_handle,
            music_sink: None,
        })
    }

    /// Load and play a MOD file from a path
    pub fn play_mod_file(&mut self, path: &Path) -> anyhow::Result<()> {
        let path_str = path.to_str().ok_or_else(|| anyhow::anyhow!("Invalid path"))?;
        let song = read_mod_file(path_str);
        let source = ModSource::new(song);

        let sink = Sink::try_new(&self.stream_handle)?;
        sink.append(source);
        self.music_sink = Some(sink);

        Ok(())
    }

    /// Play a drum hit sound
    pub fn play_hit(&self, note_type: NoteType) {
        let frequency = match note_type {
            NoteType::Don => 150.0,
            NoteType::Ka => 400.0,
        };

        let source = SineWaveSource::new(frequency, 80);

        if let Ok(sink) = Sink::try_new(&self.stream_handle) {
            sink.append(source);
            sink.detach();
        }
    }

    /// Play miss sound
    pub fn play_miss(&self) {
        let source = SineWaveSource::new(100.0, 50);

        if let Ok(sink) = Sink::try_new(&self.stream_handle) {
            sink.append(source);
            sink.detach();
        }
    }
}

/// Wrapper for mod_player as a rodio Source
struct ModSource {
    song: Song,
    player_state: PlayerState,
    current_channel: usize, // 0 = left, 1 = right
    current_sample: (f32, f32),
}

impl ModSource {
    fn new(song: Song) -> Self {
        let player_state = PlayerState::new(song.format.num_channels as u32, SAMPLE_RATE);
        ModSource {
            song,
            player_state,
            current_channel: 0,
            current_sample: (0.0, 0.0),
        }
    }
}

impl Iterator for ModSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_channel == 0 {
            // Generate new sample pair
            self.current_sample = next_sample(&self.song, &mut self.player_state);
            self.current_channel = 1;
            Some(self.current_sample.0 * 0.5) // Left channel, reduce volume
        } else {
            self.current_channel = 0;
            Some(self.current_sample.1 * 0.5) // Right channel
        }
    }
}

impl Source for ModSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        2 // Stereo
    }

    fn sample_rate(&self) -> u32 {
        SAMPLE_RATE
    }

    fn total_duration(&self) -> Option<Duration> {
        None // Unknown/looping
    }
}

/// Simple sine wave source for drum hits
struct SineWaveSource {
    frequency: f32,
    sample_rate: u32,
    duration_samples: usize,
    current_sample: usize,
}

impl SineWaveSource {
    fn new(frequency: f32, duration_ms: u64) -> Self {
        let sample_rate = 48000;
        let duration_samples = (sample_rate as u64 * duration_ms / 1000) as usize;
        SineWaveSource {
            frequency,
            sample_rate,
            duration_samples,
            current_sample: 0,
        }
    }
}

impl Iterator for SineWaveSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_sample >= self.duration_samples {
            return None;
        }

        let t = self.current_sample as f32 / self.sample_rate as f32;
        let envelope = 1.0 - (self.current_sample as f32 / self.duration_samples as f32);
        let sample = (2.0 * std::f32::consts::PI * self.frequency * t).sin() * envelope * 0.3;

        self.current_sample += 1;
        Some(sample)
    }
}

impl Source for SineWaveSource {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.duration_samples - self.current_sample)
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_millis(
            (self.duration_samples as u64 * 1000) / self.sample_rate as u64,
        ))
    }
}

//! Android live playback through AAudio (API 26+).
//!
//! A worker thread owns the stream (AAudio handles are not `Send`) and makes
//! blocking writes of interleaved stereo `f32` frames drained from the shared
//! queue, mirroring the Linux pipe backend.

use super::queue::SampleQueue;
use super::LIVE_FRAME_SAMPLES;
use ndk::audio::{
    AudioDirection, AudioFormat, AudioPerformanceMode, AudioSharingMode, AudioStreamBuilder,
};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread::{self, JoinHandle};

const SAMPLE_RATE: i32 = 44_100;
/// A write may block for at most this long before the slice is dropped.
const WRITE_TIMEOUT_NS: i64 = 100_000_000;

pub struct AudioOutput {
    queue: Arc<SampleQueue>,
    worker: Option<JoinHandle<()>>,
}

impl AudioOutput {
    /// Opens the default output device, or returns `None` when AAudio refuses.
    pub fn open() -> Option<Self> {
        let queue = Arc::new(SampleQueue::default());
        let playback = Arc::clone(&queue);
        let (ready, opened) = mpsc::channel();
        let worker = thread::Builder::new()
            .name("sml2-audio-output".into())
            .spawn(move || {
                let stream = AudioStreamBuilder::new().and_then(|builder| {
                    builder
                        .direction(AudioDirection::Output)
                        .format(AudioFormat::PCM_Float)
                        .channel_count(2)
                        .sample_rate(SAMPLE_RATE)
                        .sharing_mode(AudioSharingMode::Shared)
                        .performance_mode(AudioPerformanceMode::LowLatency)
                        .open_stream()
                });
                let stream = match stream {
                    Ok(stream) if stream.request_start().is_ok() => stream,
                    _ => {
                        let _ = ready.send(false);
                        return;
                    }
                };
                // Two ticks of headroom keeps underruns rare on slow TV sticks.
                let _ = stream.set_buffer_size_in_frames(2 * LIVE_FRAME_SAMPLES as i32);
                let _ = ready.send(true);
                let mut limited = Vec::with_capacity(2 * LIVE_FRAME_SAMPLES);
                while let Some(samples) = playback.pop() {
                    limited.clear();
                    limited.extend(samples.iter().map(|&s| super::limit(s)));
                    let mut written = 0usize;
                    while written < limited.len() / 2 {
                        let rest = &limited[written * 2..];
                        // SAFETY: `rest` holds `rest.len() / 2` interleaved stereo
                        // f32 frames, matching the stream's format and channels.
                        let result = unsafe {
                            stream.write(
                                rest.as_ptr().cast(),
                                (rest.len() / 2) as i32,
                                WRITE_TIMEOUT_NS,
                            )
                        };
                        match result {
                            Ok(0) | Err(_) => break,
                            Ok(frames) => written += frames as usize,
                        }
                    }
                }
                let _ = stream.request_stop();
            })
            .ok()?;
        if opened.recv().unwrap_or(false) {
            Some(Self {
                queue,
                worker: Some(worker),
            })
        } else {
            let _ = worker.join();
            None
        }
    }

    /// The stream is opened at 44.1 kHz; AAudio resamples if the device differs.
    pub fn sample_rate(&self) -> u32 {
        SAMPLE_RATE as u32
    }

    /// Enqueue a frame of stereo interleaved float32 samples to the audio stream.
    pub fn write_samples(&self, samples: &[f32]) {
        self.queue.push(samples.to_vec());
    }
}

impl Drop for AudioOutput {
    fn drop(&mut self) {
        self.queue.close();
        if let Some(handle) = self.worker.take() {
            let _ = handle.join();
        }
    }
}

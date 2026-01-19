//! Audio recording module for whisp.
//!
//! This crate provides audio recording functionality using the system's
//! default input device. It's platform-agnostic and uses channels for
//! event communication instead of depending on any specific UI framework.
//!
//! ## Format notes
//!
//! WAV format uses ~467KiB every 5 seconds, hitting the 25MiB API limit
//! in about 4m30s. This is sufficient for most dictation use cases.

use std::io::{self, Cursor, Seek, SeekFrom, Write};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use cpal::Host;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::{WavSpec, WavWriter};
use thiserror::Error;
use tracing::{error, info};

use crate::core::{AudioEvent, MicState, RecordingState};

/// Errors that can occur during recording.
#[derive(Debug, Error)]
pub enum RecorderError {
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),

    #[error("no input device available")]
    NoInputDevice,

    #[error("sample format not supported: {0}")]
    SampleFormatNotSupported(String),

    #[error(transparent)]
    BuildStream(#[from] cpal::BuildStreamError),
}

pub type Result<T> = std::result::Result<T, RecorderError>;

type WavWriterHandle = Arc<Mutex<Option<WavWriter<MemoryWriter>>>>;

/// A cheaply cloneable handle to the recording buffer.
#[derive(Clone)]
struct MemoryWriter {
    inner: Arc<Mutex<Cursor<Vec<u8>>>>,
}

impl MemoryWriter {
    fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Cursor::new(Vec::with_capacity(8 * 1024)))),
        }
    }

    fn try_into_inner(self) -> Result<Vec<u8>> {
        let owned = Arc::try_unwrap(self.inner).map_err(|_| {
            RecorderError::Anyhow(anyhow::anyhow!(
                "Failed to unwrap inner Arc in MemoryWriter"
            ))
        })?;
        let cursor = owned.into_inner().unwrap();
        Ok(cursor.into_inner())
    }
}

impl Seek for MemoryWriter {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.inner.lock().unwrap().seek(pos)
    }
}

impl Write for MemoryWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.lock().unwrap().flush()
    }
}

/// Audio recorder using the system's default input device.
pub struct Recorder {
    host: Host,
}

impl Default for Recorder {
    fn default() -> Self {
        Self::new()
    }
}

impl Recorder {
    /// Create a new recorder.
    pub fn new() -> Self {
        Self {
            host: cpal::default_host(),
        }
    }

    /// Start recording audio.
    ///
    /// The `event_sender` is used to notify when the mic becomes active
    /// (receives non-silent audio). Pass `None` if you don't need events.
    pub fn start_recording(
        &self,
        event_sender: Option<Sender<AudioEvent>>,
    ) -> Result<RecordingHandle> {
        let device = self
            .host
            .default_input_device()
            .ok_or(RecorderError::NoInputDevice)?;
        let config = device
            .default_input_config()
            .map_err(|_| RecorderError::NoInputDevice)?;

        info!(
            device_name = %device.name().unwrap_or_default(),
            config = ?config,
            "Recording from device"
        );

        let spec = wav_spec_from_config(&config);

        let buffer = MemoryWriter::new();
        let writer =
            WavWriter::new(buffer.clone(), spec).map_err(|e| RecorderError::Anyhow(e.into()))?;
        let writer = Arc::new(Mutex::new(Some(writer)));

        let writer_2 = writer.clone();

        let err_fn = move |err| {
            error!("an error occurred on stream: {}", err);
        };

        let mut state = RecordingState::default();

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &config.into(),
                move |data, _: &_| write_data(&mut state, data, &writer_2, &event_sender),
                err_fn,
                None,
            )?,
            sample_format => {
                return Err(RecorderError::SampleFormatNotSupported(format!(
                    "{:?}",
                    sample_format
                )));
            }
        };

        stream
            .play()
            .map_err(|_| anyhow::anyhow!("failed to play stream"))?;

        Ok(RecordingHandle {
            stream,
            writer,
            buffer: Some(buffer),
            spec,
        })
    }
}

/// Handle to an active recording.
///
/// Call `finish()` to stop recording and retrieve the audio data.
/// If dropped without calling `finish()`, the recording will be finalized
/// but you won't be able to retrieve the data.
pub struct RecordingHandle {
    stream: cpal::Stream,
    writer: WavWriterHandle,
    buffer: Option<MemoryWriter>,
    spec: WavSpec,
}

/// A completed recording with audio data.
pub struct Recording {
    data: Vec<u8>,
    spec: WavSpec,
}

impl Recording {
    /// Get the raw audio data (WAV format).
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get the WAV specification.
    pub fn spec(&self) -> &WavSpec {
        &self.spec
    }

    /// Get the number of samples in the recording.
    pub fn samples(&self) -> u64 {
        self.data.len() as u64 / (self.spec.bits_per_sample / 8) as u64
    }

    /// Get the duration of the recording.
    pub fn duration(&self) -> Duration {
        let num_samples = self.samples();
        let duration = num_samples as f64 / self.spec.sample_rate as f64;
        Duration::from_secs_f64(duration)
    }

    /// Consume the recording and return the raw data.
    pub fn into_data(self) -> Vec<u8> {
        self.data
    }
}

impl RecordingHandle {
    /// Finish the recording and return the audio data.
    pub fn finish(&mut self) -> Result<Option<Recording>> {
        if self.buffer.is_none() {
            return Ok(None);
        }

        info!("ending recording");
        let buffer = self.buffer.take().unwrap();

        self.stream.pause().ok();

        self.writer
            .lock()
            .unwrap()
            .take()
            .unwrap()
            .finalize()
            .map_err(|e| {
                RecorderError::Anyhow(anyhow::anyhow!("Failed to finalize writer: {}", e))
            })?;

        let data = buffer.try_into_inner()?;

        Ok(Some(Recording {
            data,
            spec: self.spec,
        }))
    }
}

impl Drop for RecordingHandle {
    fn drop(&mut self) {
        if self.buffer.is_some()
            && let Err(e) = self.finish()
        {
            error!("failed to finalize recording: {}", e);
        }
    }
}

fn wav_spec_from_config(config: &cpal::SupportedStreamConfig) -> hound::WavSpec {
    hound::WavSpec {
        channels: config.channels(),
        sample_rate: config.sample_rate().0,
        bits_per_sample: (config.sample_format().sample_size() * 8) as _,
        sample_format: sample_format(config.sample_format()),
    }
}

fn sample_format(format: cpal::SampleFormat) -> hound::SampleFormat {
    if format.is_float() {
        hound::SampleFormat::Float
    } else {
        hound::SampleFormat::Int
    }
}

fn write_data(
    state: &mut RecordingState,
    data: &[f32],
    writer: &WavWriterHandle,
    event_sender: &Option<Sender<AudioEvent>>,
) {
    if !state.mic_active {
        if data.iter().any(|&sample| sample != 0.0) {
            state.mic_active = true;
            if let Some(sender) = event_sender {
                sender.send(AudioEvent::StateChanged(MicState::Active)).ok();
            }
        } else {
            return;
        }
    }

    if let Ok(mut guard) = writer.try_lock()
        && let Some(writer) = guard.as_mut()
    {
        for &sample in data.iter() {
            writer.write_sample(sample).ok();
        }
    }
}

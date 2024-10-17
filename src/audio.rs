//! Module for managing audio recording. There can only be one active recording
//! at a time and storage/processes are not managed by this module.
//!
//! ## Format notes
//!
//! Wav ~ 467KiB every 5 seconds, meaning we hit our limit of 25MiB in 4m30s.
//! This is plenty of time but a lot of data regardless. Need to consider lossy
//! formats. Whisper supports: m4a, mp3, webm, mp4, mpga, wav, and mpeg.

use std::io::{self, Cursor, Seek, SeekFrom, Write};
use std::sync::{Arc, Mutex};

use anyhow::anyhow;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Host, Sample};
use hound::WavWriter;
use thiserror::Error;
use tracing::{error, info};

#[derive(Debug, Error)]
pub enum RecorderError {
    /// generic anyhow error
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
    /// No recording device available
    #[error("no input device available")]
    NoInputDevice,
    /// Sample format not supported
    #[error("sample format not supported: {0}")]
    SampleFormatNotSupported(String),
    /// Build stream error
    #[error(transparent)]
    BuildStream(#[from] cpal::BuildStreamError),
}

type Result<T> = std::result::Result<T, RecorderError>;
type WavWriterHandle = Arc<Mutex<Option<WavWriter<MemoryWriter>>>>;

/// A cheaply cloneable handle to the inner data that is being recorded. The
/// finalize method for the wav writer does not return the inner data, so we
/// store it behind an Arc<Mutex> to allow for cheap cloning and access to the
/// inner data.
#[derive(Debug, Clone)]
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
        // Attempt to own the inner arc
        let owned = Arc::try_unwrap(self.inner).map_err(|_| {
            RecorderError::Anyhow(anyhow!("Failed to unwrap inner Arc in MemoryWriter"))
        })?;
        // Extract the cursor
        let cursor = owned.into_inner().map_err(|_| {
            RecorderError::Anyhow(anyhow!("Failed to unwrap inner Mutex in MemoryWriter"))
        })?;
        // Extract the Vec
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

pub struct Recorder {
    host: Host,
}

impl Recorder {
    pub fn new() -> Self {
        Self {
            host: cpal::default_host(),
        }
    }

    pub fn start_recording(&self) -> Result<RecordingHandle> {
        let device = self
            .host
            .default_input_device()
            .ok_or(RecorderError::NoInputDevice)?;
        let config = device
            .default_input_config()
            .map_err(|_| RecorderError::NoInputDevice)?;

        info!(device_name=%device.name().unwrap(), "Recording from device");

        let spec = wav_spec_from_config(&config);

        let buffer = MemoryWriter::new();
        let writer =
            WavWriter::new(buffer.clone(), spec).map_err(|e| RecorderError::Anyhow(e.into()))?;
        let writer = Arc::new(Mutex::new(Some(writer)));

        // Run the input stream on a separate thread.
        let writer_2 = writer.clone();

        let err_fn = move |err| {
            error!("an error occurred on stream: {}", err);
        };

        let stream = match config.sample_format() {
            cpal::SampleFormat::I8 => device.build_input_stream(
                &config.into(),
                move |data, _: &_| write_input_data::<i8, i8>(data, &writer_2),
                err_fn,
                None,
            )?,
            cpal::SampleFormat::I16 => device.build_input_stream(
                &config.into(),
                move |data, _: &_| write_input_data::<i16, i16>(data, &writer_2),
                err_fn,
                None,
            )?,
            cpal::SampleFormat::I32 => device.build_input_stream(
                &config.into(),
                move |data, _: &_| write_input_data::<i32, i32>(data, &writer_2),
                err_fn,
                None,
            )?,
            cpal::SampleFormat::F32 => device.build_input_stream(
                &config.into(),
                move |data, _: &_| write_input_data::<f32, f32>(data, &writer_2),
                err_fn,
                None,
            )?,
            sample_format => {
                return Err(RecorderError::SampleFormatNotSupported(format!(
                    "{:?}",
                    sample_format
                )))
            }
        };

        stream
            .play()
            .map_err(|_| anyhow!("failed to play stream"))?;

        Ok(RecordingHandle {
            stream,
            writer,
            buffer: Some(buffer),
        })
    }
}

/// Handle to the active recording. When dropped or finalized, the recording
/// will end. You must call `finalize` to recieve the data.
pub struct RecordingHandle {
    stream: cpal::Stream,
    writer: WavWriterHandle,
    // The buffer the data is being written to. Presence of this buffer
    // indicates if the recording has been finalized or not.
    buffer: Option<MemoryWriter>,
}

impl RecordingHandle {
    pub fn finish(&mut self) -> Result<Option<Vec<u8>>> {
        if self.buffer.is_none() {
            return Ok(None);
        }
        let buffer = self.buffer.take().unwrap();
        // can not drop because we have that &mut self instead of self.
        // drop(self.stream);
        // instead: pause and ignore errors.
        self.stream.pause().ok();
        // Finalize the writer so it writes the proper framing information.
        self.writer
            .lock()
            .unwrap()
            .take()
            .unwrap()
            .finalize()
            .map_err(|e| RecorderError::Anyhow(anyhow!("Failed to finalize writer: {}", e)))?;
        // Now that its ended, we can grab out the actual data and return it.
        let data = buffer.try_into_inner()?;
        Ok(Some(data))
    }
}

impl Drop for RecordingHandle {
    fn drop(&mut self) {
        if self.buffer.is_some() {
            if let Err(e) = self.finish() {
                error!("failed to finalize recording: {}", e);
            }
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

fn write_input_data<T, U>(input: &[T], writer: &WavWriterHandle)
where
    T: Sample,
    U: Sample + hound::Sample + FromSample<T>,
{
    if let Ok(mut guard) = writer.try_lock() {
        if let Some(writer) = guard.as_mut() {
            for &sample in input.iter() {
                let sample: U = U::from_sample(sample);
                writer.write_sample(sample).ok();
            }
        }
    }
}

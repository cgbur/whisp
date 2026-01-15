//! Audio processing pipeline.
//!
//! This module handles the async processing of recorded audio,
//! including transcription and result delivery.

use std::sync::Arc;
use std::time::Instant;

use std::sync::RwLock;
use whisp_transcribe::Bytes;
use tao::event_loop::EventLoopProxy;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::event::WhispEvent;
use crate::{Config, MicState, Recording, Transcriber};

/// Processing pipeline for audio data.
pub struct AudioPipeline {
    runtime: Runtime,
    config: Arc<RwLock<Config>>,
    transcriber: Arc<dyn Transcriber>,
    transcription_handles: mpsc::UnboundedSender<TranscriptionTask>,
}

type TranscriptionTask = tokio::task::JoinHandle<TranscriptionResult>;

/// Result of submitting audio to the pipeline.
pub enum SubmitResult {
    /// Audio was sent for processing
    Sent,
    /// Audio was discarded (too short)
    Discarded,
}

impl AudioPipeline {
    /// Create a new pipeline instance.
    pub fn new(
        config: Arc<RwLock<Config>>,
        transcriber: Arc<dyn Transcriber>,
        event_sender: EventLoopProxy<WhispEvent>,
    ) -> anyhow::Result<Self> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()?;

        let transcription_handles = start_results_collector(&runtime, event_sender)?;

        info!(
            transcriber = transcriber.name(),
            "Audio pipeline initialized"
        );

        Ok(Self {
            runtime,
            config,
            transcriber,
            transcription_handles,
        })
    }

    /// Submit audio for processing.
    pub fn submit(&self, recording: Recording) -> anyhow::Result<SubmitResult> {
        info!(
            samples = recording.samples(),
            bytes = recording.data().len(),
            bytes_mb = recording.data().len() as f64 / (1024.0 * 1024.0),
            length_seconds = recording.duration().as_secs_f64(),
            "audio submitted"
        );

        if recording.duration() < self.config.read().unwrap().discard_duration() {
            info!(
                discard_duration = ?self.config.read().unwrap().discard_duration(),
                "discarding recording"
            );
            return Ok(SubmitResult::Discarded);
        }

        let transcriber = self.transcriber.clone();
        let config = self.config.clone();
        let handle = self
            .runtime
            .spawn(transcribe(transcriber, config, recording));

        self.transcription_handles.send(handle)?;
        Ok(SubmitResult::Sent)
    }
}

async fn transcribe(
    transcriber: Arc<dyn Transcriber>,
    config: Arc<RwLock<Config>>,
    recording: Recording,
) -> TranscriptionResult {
    // Bytes is reference-counted, so cloning is O(1)
    let audio: Bytes = recording.into_data().into();
    let num_bytes = audio.len();

    let (mut num_retries, language) = {
        let config_read = config.read().unwrap();
        (
            config_read.retries,
            config_read.language().map(|s| s.to_string()),
        )
    };

    let mut before = Instant::now();
    let mut result = transcriber.transcribe(audio.clone(), language.as_deref()).await;

    while result.is_err() && num_retries > 0 {
        warn!("Retrying transcription, previous error: {:?}", result);
        before = Instant::now();
        result = transcriber.transcribe(audio.clone(), language.as_deref()).await;
        num_retries -= 1;
    }

    let Ok(text) = result else {
        return TranscriptionResult::RetryError {
            retries: config.read().unwrap().retries,
            error: anyhow::anyhow!("Transcription failed"),
            data: audio.to_vec(),
        };
    };

    let duration = before.elapsed();
    let mb_per_second = num_bytes as f64 / (1024.0 * 1024.0) / duration.as_secs_f64();
    info!(
        duration = ?duration,
        mb_per_second = mb_per_second,
        "transcription completed"
    );

    TranscriptionResult::Success(text)
}

enum TranscriptionResult {
    Success(String),
    RetryError {
        retries: u8,
        error: anyhow::Error,
        data: Vec<u8>,
    },
}

fn start_results_collector(
    runtime: &Runtime,
    event_sender: EventLoopProxy<WhispEvent>,
) -> anyhow::Result<mpsc::UnboundedSender<TranscriptionTask>> {
    let (task_sender, mut task_receiver) = tokio::sync::mpsc::unbounded_channel();

    runtime.spawn(async move {
        while let Some(task) = task_receiver.recv().await {
            match task.await {
                Ok(TranscriptionResult::Success(text)) => {
                    info!("Transcription: {}", text);
                    event_sender
                        .send_event(WhispEvent::TranscriptReady(text))
                        .ok();
                }
                Ok(TranscriptionResult::RetryError {
                    retries,
                    error,
                    data,
                }) => {
                    error!(
                        "Transcription failed after {} retries: {:?}",
                        retries, error
                    );
                    event_sender
                        .send_event(WhispEvent::StateChanged(MicState::Idle))
                        .ok();
                    event_sender.send_event(WhispEvent::AudioError(data)).ok();
                }
                Err(e) => {
                    error!("Error joining audio handler: {:?}", e);
                }
            }
        }

        error!("Results collector task ended unexpectedly");
    });

    Ok(task_sender)
}

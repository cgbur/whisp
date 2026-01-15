//! Audio processing pipeline.
//!
//! This module handles the async processing of recorded audio,
//! including transcription and result delivery.

use std::sync::Arc;
use std::time::Instant;

use parking_lot::RwLock;
use tao::event_loop::EventLoopProxy;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::event::WhispEvent;
use crate::{
    Config, MicState, OpenAIClient, OpenAIConfig, Recording, TranscribeRequest, Transcriber,
};

/// Processing pipeline for audio data.
pub struct AudioPipeline {
    runtime: Runtime,
    config: Arc<RwLock<Config>>,
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
        event_sender: EventLoopProxy<WhispEvent>,
    ) -> anyhow::Result<Self> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()?;

        let transcription_handles = start_results_collector(&runtime, event_sender)?;

        Ok(Self {
            runtime,
            config,
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

        if recording.duration() < self.config.read().discard_duration() {
            info!(
                discard_duration = ?self.config.read().discard_duration(),
                "discarding recording"
            );
            return Ok(SubmitResult::Discarded);
        }

        let config = self.config.clone();
        let handle = self.runtime.spawn(transcribe(config, recording));

        self.transcription_handles.send(handle)?;
        Ok(SubmitResult::Sent)
    }
}

async fn transcribe(config: Arc<RwLock<Config>>, recording: Recording) -> TranscriptionResult {
    let audio = recording.into_data();
    let bytes = audio.len();

    let config_read = config.read();
    let mut num_retries = config_read.retries;

    // Build the transcription client
    let api_key = match config_read.key_openai() {
        Some(key) => key.to_string(),
        None => {
            return TranscriptionResult::RetryError {
                retries: 0,
                error: anyhow::anyhow!("No OpenAI API key configured"),
                data: audio,
            };
        }
    };

    let mut client_config = OpenAIConfig::new(&api_key);
    if let Some(model) = config_read.model() {
        client_config = client_config.with_model(model);
    }
    let client = OpenAIClient::new(client_config);

    let language = config_read.language().map(|s| s.to_string());
    drop(config_read);

    let request = TranscribeRequest {
        audio: audio.clone(),
        language,
        prompt: None,
    };

    let mut before = Instant::now();
    let mut result = client.transcribe(request.clone()).await;

    while result.is_err() && num_retries > 0 {
        warn!("Retrying transcription, previous error: {:?}", result);
        before = Instant::now();
        result = client.transcribe(request.clone()).await;
        num_retries -= 1;
    }

    let Ok(response) = result else {
        return TranscriptionResult::RetryError {
            retries: config.read().retries,
            error: anyhow::anyhow!("Transcription failed"),
            data: audio,
        };
    };

    let duration = before.elapsed();
    let mb_per_second = bytes as f64 / (1024.0 * 1024.0) / duration.as_secs_f64();
    info!(
        duration = ?duration,
        mb_per_second = mb_per_second,
        "transcription completed"
    );

    TranscriptionResult::Success(response.text)
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

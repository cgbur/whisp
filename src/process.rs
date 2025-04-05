use std::sync::Arc;
use std::time::Instant;

use parking_lot::RwLock;
use tao::event_loop::EventLoopProxy;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::config::Config;
use crate::event::WhispEvent;
use crate::icon::MicState;
use crate::models::ModelClient;
use crate::record::Recording;

/// Processing pipeline for audio data. This accepts audio data bytes and
/// performs the processing pipeline stages on it. Carrying it through from
/// transcription to pasting.
pub struct AudioPipeline {
    runtime: Runtime,
    model: ModelClient,
    config: Arc<RwLock<Config>>,
    transcription_handles: mpsc::UnboundedSender<TranscriptionTask>,
}

type TranscriptionTask = tokio::task::JoinHandle<TranscriptionResult>;

pub enum SubmitResult {
    Sent,
    Discarded,
}

impl AudioPipeline {
    /// Create a new pipeline instance.
    pub fn new(
        config: Arc<RwLock<Config>>,
        event_sender: EventLoopProxy<WhispEvent>,
    ) -> anyhow::Result<Self> {
        // Set up tokio runtime
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()?;

        // Client for interacting with models
        let model = ModelClient::new()?;

        // Start the results collector.
        let transcription_handles = start_results_collector(&runtime, event_sender)?;

        Ok(Self {
            runtime,
            model,
            config,
            transcription_handles,
        })
    }

    /// Submits a new audio sample to the processing pipeline. This is
    /// non-blocking and all samples will be processed in order.
    pub fn submit(&self, recording: Recording) -> anyhow::Result<SubmitResult> {
        info!(
            samples = recording.samples(),
            bytes = recording.data().len(),
            bytes_mb = recording.data().len() as f64 / (1024.0 * 1024.0),
            length_seconds = recording.duration().as_secs_f64(),
            "audio submitted"
        );

        if recording.duration() < self.config.read().discard_duration() {
            info!(discard_duration = ?self.config.read().discard_duration(), "discarding recording");
            return Ok(SubmitResult::Discarded);
        }

        let model = self.model.clone();
        let config = self.config.clone();

        // Spawn a new task to handle the transcription
        let handle = self.runtime.spawn(transcribe(model, config, recording));

        // Send the transcription task to the collector
        self.transcription_handles.send(handle)?;
        Ok(SubmitResult::Sent)
    }
}

/// Helper to call the transcription model and collect some basic stats.
async fn transcribe(
    model: ModelClient,
    config: Arc<RwLock<Config>>,
    recording: Recording,
) -> TranscriptionResult {
    let audio = recording.into_data();
    let bytes = audio.len();
    let mut num_retries = config.read().retries();

    // Send off the audio to the model for transcription
    let mut before = Instant::now();
    let mut result = model.transcribe(config.clone(), audio.clone()).await;
    while result.is_err() && num_retries > 0 {
        warn!("Retrying transcription, previous error: {:?}", result);
        before = Instant::now();
        result = model.transcribe(config.clone(), audio.clone()).await;
        num_retries -= 1;
    }
    let Ok(result) = result else {
        return TranscriptionResult::RetryError {
            retries: config.read().retries(),
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

    TranscriptionResult::Success(result)
}

enum TranscriptionResult {
    Success(String),
    RetryError {
        retries: u8,
        error: anyhow::Error,
        data: Vec<u8>,
    },
}

// Well now, let's just see how this fails.That was good.

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

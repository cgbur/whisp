use std::sync::Arc;

use anyhow::Result;
use parking_lot::RwLock;
use tao::event_loop::EventLoopProxy;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tracing::{error, info};

use crate::config::Config;
use crate::event::UserEvent;
use crate::models::ModelClient;

/// Processing pipeline for audio data. This accepts audio data bytes and
/// performs the processing pipeline stages on it. Carrying it through from
/// transcription to pasting.
pub struct Processor {
    runtime: Runtime,
    model: ModelClient,
    config: Arc<RwLock<Config>>,
    transcription_handles: mpsc::UnboundedSender<TranscriptionTask>,
}

type TranscriptionTask = tokio::task::JoinHandle<Result<String>>;

impl Processor {
    /// Create a new pipeline instance.
    pub fn new(
        config: Arc<RwLock<Config>>,
        event_sender: EventLoopProxy<UserEvent>,
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
    pub fn submit_audio(&self, audio: Vec<u8>) -> anyhow::Result<()> {
        info!(
            "Submitting audio for processing: {:.2}mb",
            audio.len() as f64 / 1024.0 / 1024.0
        );
        let model = self.model.clone();
        let config = self.config.clone();

        let handle = self
            .runtime
            .spawn(async move { model.transcribe(config, audio).await });

        self.transcription_handles.send(handle)?;
        Ok(())
    }
}

fn start_results_collector(
    runtime: &Runtime,
    event_sender: EventLoopProxy<UserEvent>,
) -> anyhow::Result<mpsc::UnboundedSender<TranscriptionTask>> {
    let (task_sender, mut task_receiver) = tokio::sync::mpsc::unbounded_channel();

    runtime.spawn(async move {
        while let Some(task) = task_receiver.recv().await {
            match task.await {
                Ok(Ok(text)) => {
                    info!("Transcription: {}", text);
                    event_sender.send_event(UserEvent::TranscriptReady(text)).ok();
                }
                Ok(Err(e)) => {
                    error!("Error processing audio: {:?}", e);
                }
                Err(e) => {
                    error!("Error processing audio: {:?}", e);
                }
            }
        }

        error!("Results collector task ended unexpectedly");
    });

    Ok(task_sender)
}

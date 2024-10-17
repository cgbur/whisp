use std::sync::{Arc, RwLock};

use anyhow::Result;
use arboard::Clipboard;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tracing::{error, info};

use crate::config::Config;
use crate::models::ModelClient;
use crate::paste::spawn_paste_task;

/// Processing pipeline for audio data. This accepts audio data bytes and
/// performs the processing pipeline stages on it. Carrying it through from
/// transcription to pasting.
pub struct Processor {
    runtime: Runtime,
    model: ModelClient,
    config: Arc<RwLock<Config>>,
    task_sender: mpsc::UnboundedSender<TranscriptionTask>,
}

type TranscriptionTask = tokio::task::JoinHandle<Result<String>>;

impl Processor {
    /// Create a new pipeline instance.
    pub fn new(config: Arc<RwLock<Config>>) -> anyhow::Result<Self> {
        // Set up tokio runtime
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()?;

        // Client for interacting with models
        let model = ModelClient::new()?;

        // Start the results collector.
        let task_sender = start_results_collector(config.clone(), &runtime)?;

        Ok(Self {
            runtime,
            model,
            config,
            task_sender,
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

        self.task_sender.send(handle)?;
        Ok(())
    }
}

fn start_results_collector(
    config: Arc<RwLock<Config>>,
    runtime: &Runtime,
) -> anyhow::Result<mpsc::UnboundedSender<TranscriptionTask>> {
    let mut clipboard = Clipboard::new()?;
    let paster = spawn_paste_task();

    let (task_sender, mut task_receiver) = tokio::sync::mpsc::unbounded_channel();

    runtime.spawn(async move {
        while let Some(task) = task_receiver.recv().await {
            match task.await {
                Ok(Ok(transcription)) => {
                    info!("Transcription: {}", transcription);
                    if handle_transcription(
                        config.clone(),
                        &mut clipboard,
                        &mut paster.clone(),
                        transcription,
                    )
                    .await
                    .is_err()
                    {
                        error!("Error handling transcription");
                    }
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

async fn handle_transcription(
    config: Arc<RwLock<Config>>,
    clipboard: &mut Clipboard,
    paster: &mut std::sync::mpsc::Sender<()>,
    transcription: String,
) -> Result<()> {
    let config = config.read().unwrap();
    info!(
        auto_paste = config.auto_paste(),
        restore_clipboard = config.restore_clipboard(),
        "Handling transcription"
    );
    let restore = config.auto_paste() && config.restore_clipboard();
    let previous = if restore {
        Some(clipboard.get_text()?)
    } else {
        None
    };

    // Copy the transcription to the clipboard
    clipboard.set_text(&transcription)?;

    if config.auto_paste() {
        // Paste the transcription
        paster.send(())?;
        if let Some(previous) = previous {
            // Restore the previous clipboard contents
            clipboard.set_text(&previous)?;
        }
    }

    Ok(())
}

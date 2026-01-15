//! OpenAI Whisper API transcription backend.

use async_trait::async_trait;
use serde::Deserialize;
use tracing::debug;

use crate::{Result, TranscribeError, Transcriber};

const TRANSCRIPTION_ENDPOINT: &str = "https://api.openai.com/v1/audio/transcriptions";
const DEFAULT_MODEL: &str = "gpt-4o-mini-transcribe";

/// Configuration for the OpenAI transcription client.
#[derive(Debug, Clone)]
pub struct OpenAIConfig {
    /// OpenAI API key
    pub api_key: String,

    /// Model to use (defaults to gpt-4o-mini-transcribe)
    pub model: Option<String>,
}

impl OpenAIConfig {
    /// Create a new OpenAI config with the given API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: None,
        }
    }

    /// Set the model to use.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Get the model name, using default if not set.
    pub fn model(&self) -> &str {
        self.model.as_deref().unwrap_or(DEFAULT_MODEL)
    }
}

/// OpenAI Whisper API client.
#[derive(Debug, Clone)]
pub struct OpenAIClient {
    client: reqwest::Client,
    config: OpenAIConfig,
}

#[derive(Debug, Deserialize)]
struct WhisperResponse {
    text: String,
}

impl OpenAIClient {
    /// Create a new OpenAI client with the given configuration.
    pub fn new(config: OpenAIConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            config,
        }
    }

    /// Create a client from just an API key with default settings.
    pub fn from_api_key(api_key: impl Into<String>) -> Self {
        Self::new(OpenAIConfig::new(api_key))
    }
}

#[async_trait]
impl Transcriber for OpenAIClient {
    async fn transcribe(&self, audio: &[u8], language: Option<&str>) -> Result<String> {
        debug!(
            model = self.config.model(),
            audio_bytes = audio.len(),
            language = ?language,
            "Sending transcription request to OpenAI"
        );

        let mut form = reqwest::multipart::Form::new()
            .part(
                "file",
                reqwest::multipart::Part::bytes(audio.to_vec())
                    .file_name("recording.wav")
                    .mime_str("audio/wav")
                    .map_err(|e| TranscribeError::ApiError(e.to_string()))?,
            )
            .part(
                "model",
                reqwest::multipart::Part::text(self.config.model().to_string()),
            );

        if let Some(lang) = language {
            form = form.part("language", reqwest::multipart::Part::text(lang.to_string()));
        }

        let response = self
            .client
            .post(TRANSCRIPTION_ENDPOINT)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(TranscribeError::ApiError(format!(
                "API returned {}: {}",
                status, body
            )));
        }

        let whisper_response: WhisperResponse = response
            .json()
            .await
            .map_err(|e| TranscribeError::TranscriptionFailed(e.to_string()))?;

        Ok(whisper_response.text)
    }

    fn name(&self) -> &str {
        "openai"
    }
}

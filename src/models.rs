use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Context;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::config::Config;

const TRANSCRIPTION_ENDPOINT: &str = "https://api.openai.com/v1/audio/transcriptions";
const DEFAULT_MODEL: &str = "gpt-4o-transcribe";

#[derive(Debug, Serialize, Clone)]
struct TranscriptionRequest {
    pub file: Vec<u8>,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct WhisperResponse {
    pub text: String,
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone)]
pub struct ModelClient {
    client: reqwest::Client,
}

impl ModelClient {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            client: reqwest::Client::new(),
        })
    }

    pub async fn transcribe(
        &self,
        config: Arc<RwLock<Config>>,
        audio: Vec<u8>,
    ) -> anyhow::Result<String> {
        let request = TranscriptionRequest {
            file: audio,
            model: config.read().model().unwrap_or(DEFAULT_MODEL).to_string(),
            prompt: None,
            response_format: None,
            temperature: None,
            language: config.read().language().map(|l| l.to_string()),
        };

        self.client
            .post(TRANSCRIPTION_ENDPOINT)
            .header(
                "Authorization",
                format!(
                    "Bearer {}",
                    config.read().key_openai().context("No OpenAI key")?
                ),
            )
            .multipart(
                reqwest::multipart::Form::new()
                    .part(
                        "file",
                        reqwest::multipart::Part::bytes(request.file)
                            .file_name("recording.wav")
                            .mime_str("audio/wav")?,
                    )
                    .part("model", reqwest::multipart::Part::text(request.model)),
            )
            .send()
            .await?
            .json::<WhisperResponse>()
            .await
            .map(|resp| resp.text)
            .context("Failed to parse Whisper response")
    }
}

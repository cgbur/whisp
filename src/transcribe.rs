use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::State;

const WHISPER_ENDPOINT: &str = "https://api.openai.com/v1/audio/transcriptions";
const DEFAULT_WHISPER_MODEL: &str = "whisper-1";

#[derive(Debug, Serialize, Clone)]
pub struct WhisperRequest {
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
pub struct WhisperResponse {
    pub text: String,
    pub headers: Option<HashMap<String, String>>,
}

pub struct Transcriber {
    client: reqwest::Client,
}

impl Transcriber {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            client: reqwest::Client::new(),
        })
    }

    pub async fn transcribe(&self, config: &Config, audio: Vec<u8>) -> anyhow::Result<String> {
        let request = WhisperRequest {
            file: audio,
            model: config.model().unwrap_or(DEFAULT_WHISPER_MODEL).to_string(),
            prompt: None,
            response_format: None,
            temperature: None,
            language: config.language().map(|l| l.to_string()),
        };

        let response = self
            .client
            .post(WHISPER_ENDPOINT)
            .header(
                "Authorization",
                format!("Bearer {}", config.key_openai().unwrap()),
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
            .await?;

        Ok(response.text)
    }
}
#[cfg(test)]
mod tests {

    use super::*;
    use crate::config::ConfigManager;

    #[test]
    fn test_openai_transcribe() {
        let config_manager = ConfigManager::new().unwrap();
        let config = config_manager.load().unwrap();
        let transcriber = Transcriber::new(Arc::new(RwLock::new(State::new().unwrap()))).unwrap();
        let audio = std::fs::read("recording.wav").unwrap();
        let text = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(transcriber.transcribe(&config, audio))
            .unwrap();
        println!("{}", text);
    }
}

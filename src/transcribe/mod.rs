//! Transcription backend library for whisp.
//!
//! This crate provides a trait-based abstraction for audio transcription,
//! with implementations for OpenAI's Whisper API and local Whisper models.

mod openai;

#[cfg(feature = "local-whisper")]
mod local;
#[cfg(feature = "local-whisper")]
mod model;

use async_trait::async_trait;
pub use bytes::Bytes;
#[cfg(feature = "local-whisper")]
pub use local::{LocalWhisperClient, LocalWhisperConfig};
#[cfg(feature = "local-whisper")]
pub use model::{WhisperModel, download_model, ensure_model, model_exists, model_path};
pub use openai::{OpenAIClient, OpenAIConfig};
use thiserror::Error;

/// Errors that can occur during transcription.
#[derive(Debug, Error)]
pub enum TranscribeError {
    #[error("API request failed: {0}")]
    ApiError(String),

    #[error("No API key configured")]
    NoApiKey,

    #[error("Invalid audio format: {0}")]
    InvalidAudioFormat(String),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Transcription failed: {0}")]
    TranscriptionFailed(String),
}

/// Result type for transcription operations.
pub type Result<T> = std::result::Result<T, TranscribeError>;

/// Trait for transcription backends.
///
/// Implement this trait to add new transcription backends (e.g., local whisper,
/// other cloud providers, etc.)
#[async_trait]
pub trait Transcriber: Send + Sync {
    /// Transcribe audio to text.
    ///
    /// # Arguments
    /// * `audio` - Raw audio data (WAV, MP3, etc.) as reference-counted bytes.
    ///             Use `Bytes::from(vec)` to convert from Vec<u8> (zero-copy).
    ///             Cloning Bytes is O(1) which allows efficient retries.
    /// * `language` - Optional language hint (ISO 639-1 code, e.g., "en")
    async fn transcribe(&self, audio: Bytes, language: Option<&str>) -> Result<String>;

    /// Returns the name of this transcriber for logging/debugging.
    fn name(&self) -> &str;
}

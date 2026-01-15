//! Transcription backend library for whisp.
//!
//! This crate provides a trait-based abstraction for audio transcription,
//! with implementations for OpenAI's Whisper API and (future) local models.

mod openai;

pub use openai::{OpenAIClient, OpenAIConfig};

use async_trait::async_trait;
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

/// Configuration for a transcription request.
#[derive(Debug, Clone, Default)]
pub struct TranscribeRequest {
    /// The audio data (WAV, MP3, etc.)
    pub audio: Vec<u8>,

    /// Optional language hint (ISO 639-1 code, e.g., "en")
    pub language: Option<String>,

    /// Optional prompt to guide transcription
    pub prompt: Option<String>,
}

/// Response from a transcription request.
#[derive(Debug, Clone)]
pub struct TranscribeResponse {
    /// The transcribed text
    pub text: String,
}

/// Trait for transcription backends.
///
/// Implement this trait to add new transcription backends (e.g., local whisper,
/// other cloud providers, etc.)
#[async_trait]
pub trait Transcriber: Send + Sync {
    /// Transcribe audio to text.
    async fn transcribe(&self, request: TranscribeRequest) -> Result<TranscribeResponse>;

    /// Returns the name of this transcriber for logging/debugging.
    fn name(&self) -> &str;
}

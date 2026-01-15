//! Model management for local Whisper transcription.
//!
//! This module handles downloading, locating, and managing Whisper models.

use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use anyhow::{Context, Result};
use tracing::{info, warn};
use whisp_core::models_dir;

/// Base URL for downloading Whisper models from Hugging Face.
const MODEL_BASE_URL: &str = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main";

/// Available Whisper model variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhisperModel {
    /// Tiny model with Q8 quantization (43.5 MB)
    TinyQ8,
    /// Tiny English-only model with Q8 quantization (43.6 MB)
    TinyEnQ8,
    /// Base model with Q8 quantization (81.8 MB) - recommended default
    BaseQ8,
    /// Base English-only model with Q8 quantization (81.8 MB)
    BaseEnQ8,
    /// Small model with Q8 quantization (264 MB)
    SmallQ8,
    /// Small English-only model with Q8 quantization (264 MB)
    SmallEnQ8,
    /// Medium model with Q8 quantization (823 MB)
    MediumQ8,
    /// Medium English-only model with Q8 quantization (823 MB)
    MediumEnQ8,
    /// Large v3 turbo model with Q5 quantization (574 MB) - best speed/quality ratio
    LargeV3TurboQ5,
}

impl WhisperModel {
    /// Returns the filename for this model.
    pub fn filename(&self) -> &'static str {
        match self {
            Self::TinyQ8 => "ggml-tiny-q8_0.bin",
            Self::TinyEnQ8 => "ggml-tiny.en-q8_0.bin",
            Self::BaseQ8 => "ggml-base-q8_0.bin",
            Self::BaseEnQ8 => "ggml-base.en-q8_0.bin",
            Self::SmallQ8 => "ggml-small-q8_0.bin",
            Self::SmallEnQ8 => "ggml-small.en-q8_0.bin",
            Self::MediumQ8 => "ggml-medium-q8_0.bin",
            Self::MediumEnQ8 => "ggml-medium.en-q8_0.bin",
            Self::LargeV3TurboQ5 => "ggml-large-v3-turbo-q5_0.bin",
        }
    }

    /// Returns the download URL for this model.
    pub fn url(&self) -> String {
        format!("{}/{}", MODEL_BASE_URL, self.filename())
    }

    /// Returns the approximate size of this model in bytes.
    pub fn size_bytes(&self) -> u64 {
        match self {
            Self::TinyQ8 => 43_500_000,
            Self::TinyEnQ8 => 43_600_000,
            Self::BaseQ8 => 81_800_000,
            Self::BaseEnQ8 => 81_800_000,
            Self::SmallQ8 => 264_000_000,
            Self::SmallEnQ8 => 264_000_000,
            Self::MediumQ8 => 823_000_000,
            Self::MediumEnQ8 => 823_000_000,
            Self::LargeV3TurboQ5 => 574_000_000,
        }
    }

    /// Returns a human-readable size string.
    pub fn size_human(&self) -> &'static str {
        match self {
            Self::TinyQ8 | Self::TinyEnQ8 => "~44 MB",
            Self::BaseQ8 | Self::BaseEnQ8 => "~82 MB",
            Self::SmallQ8 | Self::SmallEnQ8 => "~264 MB",
            Self::MediumQ8 | Self::MediumEnQ8 => "~823 MB",
            Self::LargeV3TurboQ5 => "~574 MB",
        }
    }

    /// Parses a model name string into a WhisperModel.
    ///
    /// Accepts names like "base-q8", "tiny-en-q8", "large-v3-turbo-q5", etc.
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "tiny-q8" | "tiny" => Some(Self::TinyQ8),
            "tiny-en-q8" | "tiny-en" | "tiny.en" => Some(Self::TinyEnQ8),
            "base-q8" | "base" => Some(Self::BaseQ8),
            "base-en-q8" | "base-en" | "base.en" => Some(Self::BaseEnQ8),
            "small-q8" | "small" => Some(Self::SmallQ8),
            "small-en-q8" | "small-en" | "small.en" => Some(Self::SmallEnQ8),
            "medium-q8" | "medium" => Some(Self::MediumQ8),
            "medium-en-q8" | "medium-en" | "medium.en" => Some(Self::MediumEnQ8),
            "large-v3-turbo-q5" | "large-v3-turbo" | "turbo" => Some(Self::LargeV3TurboQ5),
            _ => None,
        }
    }

    /// Returns the default model (base-q8).
    pub fn default_model() -> Self {
        Self::BaseQ8
    }
}

impl Default for WhisperModel {
    fn default() -> Self {
        Self::default_model()
    }
}

/// Returns the path where a model should be stored.
pub fn model_path(model: WhisperModel) -> Result<PathBuf> {
    Ok(models_dir()?.join(model.filename()))
}

/// Checks if a model exists locally.
pub fn model_exists(model: WhisperModel) -> Result<bool> {
    let path = model_path(model)?;
    Ok(path.exists())
}

/// Downloads a model to the local models directory.
///
/// The `progress_callback` is called periodically with (bytes_downloaded, total_bytes).
pub async fn download_model<F>(model: WhisperModel, progress_callback: F) -> Result<PathBuf>
where
    F: Fn(u64, u64) + Send + 'static,
{
    let path = model_path(model)?;

    // Create models directory if it doesn't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create models directory: {:?}", parent))?;
    }

    let url = model.url();
    info!(model = ?model, url = %url, "Downloading Whisper model");

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .with_context(|| format!("Failed to start download from {}", url))?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to download model: HTTP {}", response.status());
    }

    let total_size = response.content_length().unwrap_or(model.size_bytes());

    // Download to a temporary file first, then rename
    let temp_path = path.with_extension("bin.tmp");
    let mut file = File::create(&temp_path)
        .with_context(|| format!("Failed to create temp file: {:?}", temp_path))?;

    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    use futures_util::StreamExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.with_context(|| "Failed to read chunk during download")?;
        file.write_all(&chunk)
            .with_context(|| "Failed to write chunk to file")?;
        downloaded += chunk.len() as u64;
        progress_callback(downloaded, total_size);
    }

    file.flush().with_context(|| "Failed to flush file")?;
    drop(file);

    // Rename temp file to final path
    fs::rename(&temp_path, &path)
        .with_context(|| format!("Failed to rename {:?} to {:?}", temp_path, path))?;

    info!(path = ?path, "Model download complete");
    Ok(path)
}

/// Ensures a model is available locally, downloading it if necessary.
///
/// Returns the path to the model file.
pub async fn ensure_model<F>(model: WhisperModel, progress_callback: F) -> Result<PathBuf>
where
    F: Fn(u64, u64) + Send + 'static,
{
    if model_exists(model)? {
        info!(model = ?model, "Model already exists locally");
        return model_path(model);
    }

    warn!(
        model = ?model,
        size = model.size_human(),
        "Model not found locally, downloading..."
    );

    download_model(model, progress_callback).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_from_name() {
        assert_eq!(
            WhisperModel::from_name("base-q8"),
            Some(WhisperModel::BaseQ8)
        );
        assert_eq!(WhisperModel::from_name("base"), Some(WhisperModel::BaseQ8));
        assert_eq!(
            WhisperModel::from_name("tiny-en"),
            Some(WhisperModel::TinyEnQ8)
        );
        assert_eq!(
            WhisperModel::from_name("turbo"),
            Some(WhisperModel::LargeV3TurboQ5)
        );
        assert_eq!(WhisperModel::from_name("invalid"), None);
    }

    #[test]
    fn test_model_urls() {
        let model = WhisperModel::BaseQ8;
        assert!(model.url().contains("ggml-base-q8_0.bin"));
        assert!(model.url().starts_with("https://"));
    }
}

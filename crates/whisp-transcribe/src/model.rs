//! Model management for local Whisper transcription.
//!
//! This module handles downloading, locating, and managing Whisper models.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;

use anyhow::{Context, Result};
use tracing::{info, warn};
use whisp_core::models_dir;

/// Base URL for downloading Whisper models from Hugging Face.
const MODEL_BASE_URL: &str = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main";

macro_rules! define_models {
    (
        $(
            $variant:ident => {
                name: $name:literal,
                filename: $filename:literal,
                size_mib: $size:literal,
                sha1: $sha1:literal $(,)?
            }
        ),* $(,)?
    ) => {
        /// Available Whisper model variants.
        ///
        /// For a full list, see: <https://huggingface.co/ggerganov/whisper.cpp>
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum WhisperModel {
            $($variant),*
        }

        impl WhisperModel {
            /// Returns the config name for this model.
            pub fn name(&self) -> &'static str {
                match self {
                    $(Self::$variant => $name),*
                }
            }

            /// Returns the filename for this model.
            pub fn filename(&self) -> &'static str {
                match self {
                    $(Self::$variant => $filename),*
                }
            }

            /// Returns the expected SHA1 hash for this model.
            pub fn sha1(&self) -> &'static str {
                match self {
                    $(Self::$variant => $sha1),*
                }
            }

            /// Returns the size in MiB.
            fn size_mib(&self) -> u32 {
                match self {
                    $(Self::$variant => $size),*
                }
            }

            /// Parses a model name string into a WhisperModel.
            ///
            /// Model names must match exactly (case-insensitive).
            /// See <https://huggingface.co/ggerganov/whisper.cpp> for the full list.
            pub fn from_name(name: &str) -> Option<Self> {
                match name.to_lowercase().as_str() {
                    $($name => Some(Self::$variant)),*,
                    _ => None,
                }
            }

            /// Returns a list of all available model names.
            pub fn all_names() -> &'static [&'static str] {
                &[$($name),*]
            }
        }
    };
}

define_models! {
    Tiny => {
        name: "tiny",
        filename: "ggml-tiny.bin",
        size_mib: 75,
        sha1: "bd577a113a864445d4c299885e0cb97d4ba92b5f",
    },
    TinyQ5_1 => {
        name: "tiny-q5_1",
        filename: "ggml-tiny-q5_1.bin",
        size_mib: 31,
        sha1: "2827a03e495b1ed3048ef28a6a4620537db4ee51",
    },
    TinyQ8_0 => {
        name: "tiny-q8_0",
        filename: "ggml-tiny-q8_0.bin",
        size_mib: 42,
        sha1: "19e8118f6652a650569f5a949d962154e01571d9",
    },
    TinyEn => {
        name: "tiny.en",
        filename: "ggml-tiny.en.bin",
        size_mib: 75,
        sha1: "c78c86eb1a8faa21b369bcd33207cc90d64ae9df",
    },
    TinyEnQ5_1 => {
        name: "tiny.en-q5_1",
        filename: "ggml-tiny.en-q5_1.bin",
        size_mib: 31,
        sha1: "3fb92ec865cbbc769f08137f22470d6b66e071b6",
    },
    TinyEnQ8_0 => {
        name: "tiny.en-q8_0",
        filename: "ggml-tiny.en-q8_0.bin",
        size_mib: 42,
        sha1: "802d6668e7d411123e672abe4cb6c18f12306abb",
    },
    Base => {
        name: "base",
        filename: "ggml-base.bin",
        size_mib: 142,
        sha1: "465707469ff3a37a2b9b8d8f89f2f99de7299dac",
    },
    BaseQ5_1 => {
        name: "base-q5_1",
        filename: "ggml-base-q5_1.bin",
        size_mib: 57,
        sha1: "a3733eda680ef76256db5fc5dd9de8629e62c5e7",
    },
    BaseQ8_0 => {
        name: "base-q8_0",
        filename: "ggml-base-q8_0.bin",
        size_mib: 78,
        sha1: "7bb89bb49ed6955013b166f1b6a6c04584a20fbe",
    },
    BaseEn => {
        name: "base.en",
        filename: "ggml-base.en.bin",
        size_mib: 142,
        sha1: "137c40403d78fd54d454da0f9bd998f78703390c",
    },
    BaseEnQ5_1 => {
        name: "base.en-q5_1",
        filename: "ggml-base.en-q5_1.bin",
        size_mib: 57,
        sha1: "d26d7ce5a1b6e57bea5d0431b9c20ae49423c94a",
    },
    BaseEnQ8_0 => {
        name: "base.en-q8_0",
        filename: "ggml-base.en-q8_0.bin",
        size_mib: 78,
        sha1: "bb1574182e9b924452bf0cd1510ac034d323e948",
    },
    Small => {
        name: "small",
        filename: "ggml-small.bin",
        size_mib: 466,
        sha1: "55356645c2b361a969dfd0ef2c5a50d530afd8d5",
    },
    SmallQ5_1 => {
        name: "small-q5_1",
        filename: "ggml-small-q5_1.bin",
        size_mib: 181,
        sha1: "6fe57ddcfdd1c6b07cdcc73aaf620810ce5fc771",
    },
    SmallQ8_0 => {
        name: "small-q8_0",
        filename: "ggml-small-q8_0.bin",
        size_mib: 252,
        sha1: "bcad8a2083f4e53d648d586b7dbc0cd673d8afad",
    },
    SmallEn => {
        name: "small.en",
        filename: "ggml-small.en.bin",
        size_mib: 466,
        sha1: "db8a495a91d927739e50b3fc1cc4c6b8f6c2d022",
    },
    SmallEnQ5_1 => {
        name: "small.en-q5_1",
        filename: "ggml-small.en-q5_1.bin",
        size_mib: 181,
        sha1: "20f54878d608f94e4a8ee3ae56016571d47cba34",
    },
    SmallEnQ8_0 => {
        name: "small.en-q8_0",
        filename: "ggml-small.en-q8_0.bin",
        size_mib: 252,
        sha1: "9d75ff4ccfa0a8217870d7405cf8cef0a5579852",
    },
    SmallEnTdrz => {
        name: "small.en-tdrz",
        filename: "ggml-small.en-tdrz.bin",
        size_mib: 465,
        sha1: "b6c6e7e89af1a35c08e6de56b66ca6a02a2fdfa1",
    },
    Medium => {
        name: "medium",
        filename: "ggml-medium.bin",
        size_mib: 1536,
        sha1: "fd9727b6e1217c2f614f9b698455c4ffd82463b4",
    },
    MediumQ5_0 => {
        name: "medium-q5_0",
        filename: "ggml-medium-q5_0.bin",
        size_mib: 514,
        sha1: "7718d4c1ec62ca96998f058114db98236937490e",
    },
    MediumQ8_0 => {
        name: "medium-q8_0",
        filename: "ggml-medium-q8_0.bin",
        size_mib: 785,
        sha1: "e66645948aff4bebbec71b3485c576f3d63af5d6",
    },
    MediumEn => {
        name: "medium.en",
        filename: "ggml-medium.en.bin",
        size_mib: 1536,
        sha1: "8c30f0e44ce9560643ebd10bbe50cd20eafd3723",
    },
    MediumEnQ5_0 => {
        name: "medium.en-q5_0",
        filename: "ggml-medium.en-q5_0.bin",
        size_mib: 514,
        sha1: "bb3b5281bddd61605d6fc76bc5b92d8f20284c3b",
    },
    MediumEnQ8_0 => {
        name: "medium.en-q8_0",
        filename: "ggml-medium.en-q8_0.bin",
        size_mib: 785,
        sha1: "b1cf48c12c807e14881f634fb7b6c6ca867f6b38",
    },
    LargeV1 => {
        name: "large-v1",
        filename: "ggml-large-v1.bin",
        size_mib: 2969,
        sha1: "b1caaf735c4cc1429223d5a74f0f4d0b9b59a299",
    },
    LargeV2 => {
        name: "large-v2",
        filename: "ggml-large-v2.bin",
        size_mib: 2969,
        sha1: "0f4c8e34f21cf1a914c59d8b3ce882345ad349d6",
    },
    LargeV2Q5_0 => {
        name: "large-v2-q5_0",
        filename: "ggml-large-v2-q5_0.bin",
        size_mib: 1126,
        sha1: "00e39f2196344e901b3a2bd5814807a769bd1630",
    },
    LargeV2Q8_0 => {
        name: "large-v2-q8_0",
        filename: "ggml-large-v2-q8_0.bin",
        size_mib: 1536,
        sha1: "da97d6ca8f8ffbeeb5fd147f79010eeea194ba38",
    },
    LargeV3 => {
        name: "large-v3",
        filename: "ggml-large-v3.bin",
        size_mib: 2969,
        sha1: "ad82bf6a9043ceed055076d0fd39f5f186ff8062",
    },
    LargeV3Q5_0 => {
        name: "large-v3-q5_0",
        filename: "ggml-large-v3-q5_0.bin",
        size_mib: 1126,
        sha1: "e6e2ed78495d403bef4b7cff42ef4aaadcfea8de",
    },
    LargeV3Turbo => {
        name: "large-v3-turbo",
        filename: "ggml-large-v3-turbo.bin",
        size_mib: 1536,
        sha1: "4af2b29d7ec73d781377bfd1758ca957a807e941",
    },
    LargeV3TurboQ5_0 => {
        name: "large-v3-turbo-q5_0",
        filename: "ggml-large-v3-turbo-q5_0.bin",
        size_mib: 547,
        sha1: "e050f7970618a659205450ad97eb95a18d69c9ee",
    },
    LargeV3TurboQ8_0 => {
        name: "large-v3-turbo-q8_0",
        filename: "ggml-large-v3-turbo-q8_0.bin",
        size_mib: 834,
        sha1: "01bf15bedffe9f39d65c1b6ff9b687ea91f59e0e",
    },
}

impl WhisperModel {
    /// Returns the download URL for this model.
    pub fn url(&self) -> String {
        format!("{}/{}", MODEL_BASE_URL, self.filename())
    }

    /// Returns the approximate size of this model in bytes.
    pub fn size_bytes(&self) -> u64 {
        self.size_mib() as u64 * 1024 * 1024
    }

    /// Returns a human-readable size string.
    pub fn size_human(&self) -> String {
        let mib = self.size_mib();
        if mib >= 1024 {
            format!("{:.1} GiB", mib as f64 / 1024.0)
        } else {
            format!("{} MiB", mib)
        }
    }
}

#[allow(clippy::derivable_impls)] // Default is BaseEnQ8_0, not the first variant
impl Default for WhisperModel {
    fn default() -> Self {
        Self::BaseEnQ8_0
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

/// Computes the SHA1 hash of a file.
fn compute_sha1(path: &PathBuf) -> Result<String> {
    use sha1::{Digest, Sha1};

    let mut file = File::open(path).with_context(|| format!("Failed to open {:?}", path))?;
    let mut hasher = Sha1::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file
            .read(&mut buffer)
            .with_context(|| "Failed to read file for SHA1")?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Verifies the SHA1 hash of a downloaded model.
pub fn verify_model(model: WhisperModel) -> Result<bool> {
    let path = model_path(model)?;
    if !path.exists() {
        return Ok(false);
    }

    let expected = model.sha1();
    let actual = compute_sha1(&path)?;

    Ok(expected == actual)
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

    // Verify SHA1 before renaming
    info!("Verifying SHA1 hash...");
    let expected = model.sha1();
    let actual = compute_sha1(&temp_path)?;

    if expected != actual {
        // Remove the corrupted file
        let _ = fs::remove_file(&temp_path);
        anyhow::bail!(
            "SHA1 mismatch for {}: expected {}, got {}",
            model.filename(),
            expected,
            actual
        );
    }

    // Rename temp file to final path
    fs::rename(&temp_path, &path)
        .with_context(|| format!("Failed to rename {:?} to {:?}", temp_path, path))?;

    info!(path = ?path, "Model download complete and verified");
    Ok(path)
}

/// Ensures a model is available locally, downloading it if necessary.
///
/// Returns the path to the model file.
pub async fn ensure_model<F>(model: WhisperModel, progress_callback: F) -> Result<PathBuf>
where
    F: Fn(u64, u64) + Send + 'static,
{
    let path = model_path(model)?;

    if path.exists() {
        // Verify existing model
        info!(model = ?model, "Model exists, verifying SHA1...");
        if verify_model(model)? {
            info!(model = ?model, "Model verified");
            return Ok(path);
        }
        warn!(model = ?model, "Model SHA1 mismatch, re-downloading...");
        let _ = fs::remove_file(&path);
    }

    warn!(
        model = ?model,
        size = %model.size_human(),
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
            WhisperModel::from_name("base-q8_0"),
            Some(WhisperModel::BaseQ8_0)
        );
        assert_eq!(WhisperModel::from_name("base"), Some(WhisperModel::Base));
        assert_eq!(
            WhisperModel::from_name("tiny.en"),
            Some(WhisperModel::TinyEn)
        );
        assert_eq!(
            WhisperModel::from_name("large-v3-turbo"),
            Some(WhisperModel::LargeV3Turbo)
        );
        assert_eq!(WhisperModel::from_name("invalid"), None);
    }

    #[test]
    fn test_model_urls() {
        let model = WhisperModel::BaseQ8_0;
        assert!(model.url().contains("ggml-base-q8_0.bin"));
        assert!(model.url().starts_with("https://"));
    }

    #[test]
    fn test_all_names_parse() {
        for name in WhisperModel::all_names() {
            assert!(
                WhisperModel::from_name(name).is_some(),
                "Failed to parse model name: {}",
                name
            );
        }
    }

    #[test]
    fn test_default_model() {
        assert_eq!(WhisperModel::default(), WhisperModel::BaseQ8_0);
    }
}

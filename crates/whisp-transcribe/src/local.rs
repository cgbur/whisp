//! Local Whisper transcription using whisper-rs.
//!
//! This module provides local transcription using the whisper.cpp library
//! via whisper-rs bindings.

use std::path::PathBuf;
use std::sync::Mutex;

use async_trait::async_trait;
use tracing::{debug, info};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use crate::model::{WhisperModel, model_path};
use crate::{Result, TranscribeError, Transcriber};

/// Configuration for the local Whisper transcriber.
#[derive(Debug, Clone)]
pub struct LocalWhisperConfig {
    /// The model to use.
    pub model: WhisperModel,
    /// Optional override path to the model file.
    pub model_path: Option<PathBuf>,
}

impl Default for LocalWhisperConfig {
    fn default() -> Self {
        Self {
            model: WhisperModel::default(),
            model_path: None,
        }
    }
}

impl LocalWhisperConfig {
    /// Create a new config with the specified model.
    pub fn new(model: WhisperModel) -> Self {
        Self {
            model,
            model_path: None,
        }
    }

    /// Create a config with a custom model path.
    pub fn with_model_path(mut self, path: PathBuf) -> Self {
        self.model_path = Some(path);
        self
    }
}

/// Local Whisper transcriber using whisper.cpp.
pub struct LocalWhisperClient {
    config: LocalWhisperConfig,
    /// Lazily initialized whisper context.
    context: Mutex<Option<WhisperContext>>,
}

impl LocalWhisperClient {
    /// Create a new local Whisper client.
    pub fn new(config: LocalWhisperConfig) -> Self {
        Self {
            config,
            context: Mutex::new(None),
        }
    }

    /// Get or initialize the whisper context, returning a guard.
    fn ensure_context(&self) -> Result<std::sync::MutexGuard<'_, Option<WhisperContext>>> {
        let mut guard = self.context.lock().map_err(|e| {
            TranscribeError::TranscriptionFailed(format!("Failed to lock context: {}", e))
        })?;
        if guard.is_none() {
            let path = match &self.config.model_path {
                Some(p) => p.clone(),
                None => model_path(self.config.model)
                    .map_err(|e| TranscribeError::TranscriptionFailed(e.to_string()))?,
            };

            info!(path = ?path, "Loading Whisper model");

            let ctx = WhisperContext::new_with_params(
                path.to_str().ok_or_else(|| {
                    TranscribeError::TranscriptionFailed("Invalid model path".to_string())
                })?,
                WhisperContextParameters::default(),
            )
            .map_err(|e| {
                TranscribeError::TranscriptionFailed(format!("Failed to load model: {}", e))
            })?;

            info!("Whisper model loaded successfully");
            *guard = Some(ctx);
        }
        Ok(guard)
    }

    /// Convert WAV audio data to 16kHz mono f32 samples.
    fn convert_audio(&self, audio: &[u8]) -> Result<Vec<f32>> {
        use std::io::Cursor;

        let cursor = Cursor::new(audio);
        let reader = hound::WavReader::new(cursor).map_err(|e| {
            TranscribeError::InvalidAudioFormat(format!("Failed to read WAV: {}", e))
        })?;

        let spec = reader.spec();
        let sample_rate = spec.sample_rate;
        let channels = spec.channels as usize;

        debug!(
            sample_rate = sample_rate,
            channels = channels,
            bits_per_sample = spec.bits_per_sample,
            "Converting audio"
        );

        // Read samples as f32
        let samples: Vec<f32> = match spec.sample_format {
            hound::SampleFormat::Float => reader
                .into_samples::<f32>()
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(|e| {
                    TranscribeError::InvalidAudioFormat(format!("Failed to read samples: {}", e))
                })?,
            hound::SampleFormat::Int => {
                let bits = spec.bits_per_sample;
                let max_val = (1u32 << (bits - 1)) as f32;
                reader
                    .into_samples::<i32>()
                    .collect::<std::result::Result<Vec<_>, _>>()
                    .map_err(|e| {
                        TranscribeError::InvalidAudioFormat(format!(
                            "Failed to read samples: {}",
                            e
                        ))
                    })?
                    .into_iter()
                    .map(|s| s as f32 / max_val)
                    .collect()
            }
        };

        let original_sample_count = samples.len();

        // Convert to mono if stereo
        let mono_samples: Vec<f32> = if channels > 1 {
            samples
                .chunks(channels)
                .map(|chunk| chunk.iter().sum::<f32>() / channels as f32)
                .collect()
        } else {
            samples
        };

        // Resample to 16kHz if needed
        let target_rate = 16000;
        let resampled = if sample_rate != target_rate {
            resample(&mono_samples, sample_rate, target_rate)
        } else {
            mono_samples
        };

        debug!(
            original_samples = original_sample_count,
            resampled_samples = resampled.len(),
            "Audio conversion complete"
        );

        Ok(resampled)
    }
}

/// Simple linear interpolation resampling.
fn resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate {
        return samples.to_vec();
    }

    let ratio = from_rate as f64 / to_rate as f64;
    let new_len = (samples.len() as f64 / ratio) as usize;
    let mut result = Vec::with_capacity(new_len);

    for i in 0..new_len {
        let src_idx = i as f64 * ratio;
        let src_idx_floor = src_idx.floor() as usize;
        let frac = src_idx - src_idx_floor as f64;

        let sample = if src_idx_floor + 1 < samples.len() {
            let s0 = samples[src_idx_floor] as f64;
            let s1 = samples[src_idx_floor + 1] as f64;
            (s0 * (1.0 - frac) + s1 * frac) as f32
        } else if src_idx_floor < samples.len() {
            samples[src_idx_floor]
        } else {
            0.0
        };

        result.push(sample);
    }

    result
}

#[async_trait]
impl Transcriber for LocalWhisperClient {
    async fn transcribe(&self, audio: &[u8], language: Option<&str>) -> Result<String> {
        // Convert audio to the format whisper expects (this is CPU work, do it outside spawn_blocking)
        let samples = self.convert_audio(audio)?;
        let language = language.map(|s| s.to_string());

        // Get the context (ensures model is loaded)
        let context = self.ensure_context()?;
        let ctx = context.as_ref().expect("context should be initialized");

        // Create a new state for this transcription
        let mut state = ctx.create_state().map_err(|e| {
            TranscribeError::TranscriptionFailed(format!("Failed to create state: {}", e))
        })?;

        // Configure transcription parameters
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        // Set language if provided
        if let Some(ref lang) = language {
            params.set_language(Some(lang));
        } else {
            // Auto-detect language
            params.set_language(None);
        }

        // Disable printing to stdout
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        // Run transcription
        state.full(params, &samples).map_err(|e| {
            TranscribeError::TranscriptionFailed(format!("Transcription failed: {}", e))
        })?;

        // Collect all segments into the result
        let num_segments = state.full_n_segments().map_err(|e| {
            TranscribeError::TranscriptionFailed(format!("Failed to get segments: {}", e))
        })?;

        let mut result = String::new();
        for i in 0..num_segments {
            let segment = state.full_get_segment_text(i).map_err(|e| {
                TranscribeError::TranscriptionFailed(format!("Failed to get segment {}: {}", i, e))
            })?;
            result.push_str(&segment);
        }

        Ok(result.trim().to_string())
    }

    fn name(&self) -> &str {
        "local-whisper"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resample() {
        // Simple test: downsampling should produce fewer samples
        let samples: Vec<f32> = (0..48000).map(|i| (i as f32 / 48000.0).sin()).collect();
        let resampled = resample(&samples, 48000, 16000);
        assert_eq!(resampled.len(), 16000);
    }

    #[test]
    fn test_config_default() {
        let config = LocalWhisperConfig::default();
        assert_eq!(config.model, WhisperModel::BaseQ8);
        assert!(config.model_path.is_none());
    }
}

//! Configuration management for whisp.
//!
//! This module provides core configuration that doesn't depend on
//! platform-specific UI libraries.

use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::APP_NAME;

/// Core configuration structure for the application.
///
/// This contains settings that are platform-agnostic. Platform-specific
/// settings like hotkeys are handled separately by the main application.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// OpenAI API key
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub openai_key: Option<String>,

    /// Preferred language for transcription (ISO 639-1 code)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// Model to use for transcriptions
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Restore the clipboard contents after pasting
    #[serde(default, skip_serializing_if = "is_false")]
    pub restore_clipboard: bool,

    /// Paste contents automatically after transcribing
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub auto_paste: bool,

    /// Discard recordings under this duration (in seconds)
    #[serde(
        default = "default_discard_duration",
        skip_serializing_if = "is_default_discard_duration"
    )]
    pub discard_duration: f32,

    /// Number of retries for failed transcription requests
    #[serde(
        default = "default_retries",
        skip_serializing_if = "is_default_retries"
    )]
    pub retries: u8,

    /// Hotkey configuration (stored as string, parsed by app)
    /// Format: "modifier+modifier+key" e.g., "meta+shift+semicolon"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hotkey: Option<String>,
}

fn default_true() -> bool {
    true
}

fn is_true(v: &bool) -> bool {
    *v
}

fn is_false(v: &bool) -> bool {
    !*v
}

fn default_discard_duration() -> f32 {
    0.5
}

fn is_default_discard_duration(v: &f32) -> bool {
    (*v - 0.5).abs() < f32::EPSILON
}

fn default_retries() -> u8 {
    5
}

fn is_default_retries(v: &u8) -> bool {
    *v == 5
}

impl Default for Config {
    fn default() -> Self {
        Self {
            openai_key: None,
            language: None,
            model: None,
            restore_clipboard: false,
            auto_paste: true,
            discard_duration: default_discard_duration(),
            retries: default_retries(),
            hotkey: None,
        }
    }
}

impl Config {
    /// Get the OpenAI API key
    pub fn key_openai(&self) -> Option<&str> {
        self.openai_key.as_deref()
    }

    /// Get the preferred language
    pub fn language(&self) -> Option<&str> {
        self.language.as_deref()
    }

    /// Get the model name
    pub fn model(&self) -> Option<&str> {
        self.model.as_deref()
    }

    /// Get the discard duration as a Duration
    pub fn discard_duration(&self) -> Duration {
        Duration::from_secs_f32(self.discard_duration)
    }
}

/// Manages loading and saving configuration files.
pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    /// Creates a new ConfigManager with the default configuration directory.
    pub fn new() -> Result<Self> {
        let config_path = Self::default_config_path()?;
        Ok(Self { config_path })
    }

    /// Creates a new ConfigManager with a specified configuration directory.
    #[cfg(test)]
    pub fn with_config_dir<P: AsRef<std::path::Path>>(dir: P) -> Self {
        let config_path = dir.as_ref().join(format!("{}.toml", APP_NAME));
        Self { config_path }
    }

    /// Returns the default path to the configuration file.
    pub fn default_config_path() -> Result<PathBuf> {
        let config_dir = config_dir().context("Failed to retrieve configuration directory")?;
        Ok(config_dir.join("whisp").join(format!("{}.toml", APP_NAME)))
    }

    /// Loads the configuration from the config file or returns default.
    pub fn load(&self) -> Result<Config> {
        if !self.config_path.exists() {
            return Ok(Config::default());
        }

        let config_content = fs::read_to_string(&self.config_path)
            .with_context(|| format!("Failed to read config file at {:?}", self.config_path))?;

        let config: Config = toml::from_str(&config_content)
            .with_context(|| format!("Failed to parse config file at {:?}", self.config_path))?;

        if config.key_openai().is_none() {
            warn!(
                "OpenAI API key is not set. Transcriptions will not work without it. \
                 Copy the config path via the tray icon to set the key."
            );
        }

        Ok(config)
    }

    /// Saves the configuration to the config file.
    pub fn save(&self, config: &Config) -> Result<()> {
        let config_dir = self
            .config_path
            .parent()
            .with_context(|| format!("Failed to get parent directory of {:?}", self.config_path))?;

        fs::create_dir_all(config_dir)
            .with_context(|| format!("Failed to create config directory at {:?}", config_dir))?;

        let serialized =
            toml::to_string_pretty(&config).context("Failed to serialize configuration")?;

        fs::write(&self.config_path, serialized)
            .with_context(|| format!("Failed to write config file at {:?}", self.config_path))?;

        Ok(())
    }

    /// Returns the path to the configuration file.
    pub fn config_path(&self) -> &std::path::Path {
        &self.config_path
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.openai_key.is_none());
        assert!(config.auto_paste);
        assert!(!config.restore_clipboard);
        assert_eq!(config.retries, 5);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config {
            openai_key: Some("test-key".to_string()),
            model: Some("whisper-1".to_string()),
            ..Default::default()
        };

        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();

        assert_eq!(config.openai_key, deserialized.openai_key);
        assert_eq!(config.model, deserialized.model);
    }

    #[test]
    fn test_config_manager_save_load() {
        let temp_dir = std::env::temp_dir().join("whisp-test");
        fs::create_dir_all(&temp_dir).unwrap();

        let manager = ConfigManager::with_config_dir(&temp_dir);

        let config = Config {
            openai_key: Some("test-key".to_string()),
            ..Default::default()
        };

        manager.save(&config).unwrap();
        let loaded = manager.load().unwrap();

        assert_eq!(config.openai_key, loaded.openai_key);

        // Cleanup
        fs::remove_dir_all(&temp_dir).ok();
    }
}

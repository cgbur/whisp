//! Module for accessing, saving, and loading configuration files.

use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use dirs::config_dir;
use global_hotkey::hotkey::{HotKey, Modifiers};
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::APP_NAME;

/// Configuration structure for the application.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// The global hotkey configuration.
    #[serde(
        default = "default_hotkey",
        skip_serializing_if = "Config::is_default_hotkey"
    )]
    hotkey: HotKey,

    /// OpenAI API key. Should likely not storing this in plain text. However,
    /// if you're concern is someone having arbitrary read to your app files,
    /// you have bigger problems.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    openai_key: Option<String>,

    // Whisper settings, refactor when we support multiple models
    /// Preferred language
    #[serde(default, skip_serializing_if = "Option::is_none")]
    language: Option<String>,

    /// Model to use for transcriptions
    #[serde(default, skip_serializing_if = "Option::is_none")]
    model: Option<String>,

    /// Restore the clipboard contents after pasting. This only takes effect
    /// when we are using the auto-paste feature.
    #[serde(default, skip_serializing_if = "Config::is_default_restore_clipboard")]
    restore_clipboard: bool,

    /// Paste contents automatically after transcribing
    #[serde(
        default = "default_auto_paste",
        skip_serializing_if = "Config::is_default_auto_paste"
    )]
    auto_paste: bool,

    /// Discard recordings under a certain duration
    #[serde(
        default = "default_discard_duration",
        skip_serializing_if = "Config::is_default_discard_duration"
    )]
    discard_duration: f32,
}

impl PartialEq for Config {
    fn eq(&self, other: &Self) -> bool {
        self.hotkey == other.hotkey && self.openai_key == other.openai_key
    }
}

/// Provides the default `HotKey` configuration.
fn default_hotkey() -> HotKey {
    HotKey::new(
        Some(Modifiers::META | Modifiers::SHIFT),
        global_hotkey::hotkey::Code::Semicolon,
    )
}

fn default_auto_paste() -> bool {
    true
}

fn default_discard_duration() -> f32 {
    0.5
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hotkey: HotKey::new(
                Some(Modifiers::META | Modifiers::SHIFT),
                global_hotkey::hotkey::Code::Semicolon,
            ),
            openai_key: None,
            language: None,
            model: None,
            restore_clipboard: false,
            auto_paste: default_auto_paste(),
            discard_duration: default_discard_duration(),
        }
    }
}

impl Config {
    /// Returns the current hotkey configuration.
    pub fn hotkey(&self) -> HotKey {
        self.hotkey
    }

    /// Sets a new OpenAI API key and marks the configuration as modified.
    #[allow(unused)]
    pub fn set_key_openai(&mut self, key: &str) {
        self.openai_key = Some(key.to_owned());
    }

    /// Retrieves the OpenAI API key, if set.
    pub fn key_openai(&self) -> Option<&str> {
        self.openai_key.as_deref()
    }

    /// Checks if the provided hotkey is the default value.
    fn is_default_hotkey(hotkey: &HotKey) -> bool {
        hotkey == &Self::default().hotkey
    }

    /// Checks if the provided restore clipboard is the default value.
    fn is_default_restore_clipboard(restore_clipboard: &bool) -> bool {
        restore_clipboard == &Self::default().restore_clipboard
    }

    /// Checks if the provided auto paste is the default value.
    fn is_default_auto_paste(auto_paste: &bool) -> bool {
        auto_paste == &Self::default().auto_paste
    }

    /// Checks if the provided discard duration is the default value.
    fn is_default_discard_duration(discard_duration: &f32) -> bool {
        discard_duration == &Self::default().discard_duration
    }

    /// Returns the language configuration.
    pub fn language(&self) -> Option<&str> {
        self.language.as_deref()
    }

    /// Returns the model to use for transcriptions.
    pub fn model(&self) -> Option<&str> {
        self.model.as_deref()
    }

    /// Restore the clipboard contents after pasting. This only takes effect
    /// when we are using the auto-paste feature.
    pub fn restore_clipboard(&self) -> bool {
        self.restore_clipboard
    }

    /// Paste contents automatically after transcribing
    pub fn auto_paste(&self) -> bool {
        self.auto_paste
    }

    /// Discard recordings under a certain duration
    pub fn discard_duration(&self) -> Duration {
        Duration::from_secs_f32(self.discard_duration)
    }
}

/// Manages loading, saving, and reloading the configuration.
pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    /// Creates a new `ConfigManager` with the default configuration directory.
    pub fn new() -> Result<Self> {
        let config_path = Self::default_config_path()?;
        Ok(Self { config_path })
    }

    /// Creates a new `ConfigManager` with a specified configuration directory.
    /// Useful for testing with temporary directories.
    #[cfg(test)]
    pub fn with_config_dir<P: AsRef<std::path::Path>>(dir: P) -> Self {
        let config_path = dir.as_ref().join(format!("{}.toml", APP_NAME));
        Self { config_path }
    }

    /// Determines the default path to the configuration file using `dirs::config_dir`.
    pub fn default_config_path() -> Result<PathBuf> {
        let config_dir = config_dir().context("Failed to retrieve configuration directory")?;
        Ok(config_dir.join("whisp").join(format!("{}.toml", APP_NAME)))
    }

    /// Loads the configuration from the config file or returns the default configuration.
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

    /// Reloads the configuration and returns `true` if there are changes.
    #[cfg(test)]
    pub fn reload(&self, current_config: &mut Config) -> Result<bool> {
        let old_config = current_config.clone();
        *current_config = self.load()?;
        Ok(*current_config != old_config)
    }

    /// Saves the configuration to the config file, only writing non-default fields.
    pub fn save(&self, config: &Config) -> Result<()> {
        let config_dir = self
            .config_path
            .parent()
            .with_context(|| format!("Failed to get parent directory of {:?}", self.config_path))?;

        // Ensure the configuration directory exists.
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
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_load_default_config() {
        let temp = tempdir().expect("Failed to create temp dir");
        let manager = ConfigManager::with_config_dir(temp.path());
        let config = manager.load().unwrap();
        assert_eq!(config, Config::default());
    }

    #[test]
    fn test_save_and_load_config() {
        let temp = tempdir().expect("Failed to create temp dir");
        let manager = ConfigManager::with_config_dir(temp.path());

        let mut config = Config::default();
        config.set_key_openai("test_key");
        manager.save(&config).unwrap();

        let loaded_config = manager.load().unwrap();
        assert_eq!(loaded_config.openai_key, Some("test_key".to_string()));
        assert_eq!(loaded_config.hotkey, Config::default().hotkey);
    }

    #[test]
    fn test_reload_config() {
        let temp = tempdir().expect("Failed to create temp dir");
        let manager = ConfigManager::with_config_dir(temp.path());

        // Load the default configuration.
        let mut config = manager.load().unwrap();
        assert_eq!(config, Config::default());

        // Initially, reload should detect no changes.
        assert!(!manager.reload(&mut config).unwrap());

        // Simulate an external change by directly modifying the config file.
        let external_config = Config {
            hotkey: HotKey::new(Some(Modifiers::CONTROL), global_hotkey::hotkey::Code::KeyA),
            openai_key: Some("external_key".to_string()),
            language: Some("en".to_string()),
            model: Some("something-else".to_string()),
            restore_clipboard: true,
            auto_paste: true,
        };
        let serialized =
            toml::to_string_pretty(&external_config).expect("Failed to serialize external config");
        fs::write(manager.config_path(), serialized).expect("Failed to write external config");

        // Reload should now detect the external changes.
        let changes_detected = manager.reload(&mut config).unwrap();
        assert!(changes_detected, "Reload did not detect external changes");

        // Verify that the in-memory config matches the external changes.
        assert_eq!(config, external_config);
    }

    #[test]
    fn test_save_creates_config_file() {
        let temp = tempdir().expect("Failed to create temp dir");
        let manager = ConfigManager::with_config_dir(temp.path());

        let config = Config::default();
        manager.save(&config).unwrap();

        assert!(manager.config_path().exists());
    }

    #[test]
    fn test_set_and_get_open_ai_key() {
        let mut config = Config::default();
        assert!(config.key_openai().is_none());

        config.set_key_openai("my_api_key");
        assert_eq!(config.key_openai(), Some("my_api_key"));
    }

    #[test]
    fn test_is_default_hotkey() {
        let default_hotkey = Config::default().hotkey();
        assert!(Config::is_default_hotkey(&default_hotkey));

        let mut custom_hotkey = default_hotkey;
        custom_hotkey.key = global_hotkey::hotkey::Code::KeyA;
        assert!(!Config::is_default_hotkey(&custom_hotkey));
    }
}

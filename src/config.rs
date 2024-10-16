//! Module for accessing, saving, and loading configuration files.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use dirs::config_dir;
use global_hotkey::hotkey::{HotKey, Modifiers};
use serde::{Deserialize, Serialize};

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
    open_ai_key: Option<String>,

    /// Preferred language
    #[serde(default, skip_serializing_if = "Option::is_none")]
    language: Option<String>,
}

impl PartialEq for Config {
    fn eq(&self, other: &Self) -> bool {
        self.hotkey == other.hotkey && self.open_ai_key == other.open_ai_key
    }
}

/// Provides the default `HotKey` configuration.
fn default_hotkey() -> HotKey {
    HotKey::new(
        Some(Modifiers::META | Modifiers::SHIFT),
        global_hotkey::hotkey::Code::Semicolon,
    )
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hotkey: HotKey::new(
                Some(Modifiers::META | Modifiers::SHIFT),
                global_hotkey::hotkey::Code::Semicolon,
            ),
            open_ai_key: None,
            language: None,
        }
    }
}

impl Config {
    /// Returns the current hotkey configuration.
    pub fn hotkey(&self) -> HotKey {
        self.hotkey
    }

    /// Sets a new OpenAI API key and marks the configuration as modified.
    pub fn set_key_openai(&mut self, key: &str) {
        self.open_ai_key = Some(key.to_owned());
    }

    /// Retrieves the OpenAI API key, if set.
    pub fn key_openai(&self) -> Option<&str> {
        self.open_ai_key.as_deref()
    }

    /// Checks if the provided hotkey is the default value.
    fn is_default_hotkey(hotkey: &HotKey) -> bool {
        hotkey == &Self::default().hotkey
    }

    /// Returns the language configuration.
    pub fn language(&self) -> Option<&str> {
        self.language.as_deref()
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
    pub fn with_config_dir<P: AsRef<Path>>(dir: P) -> Self {
        let config_path = dir.as_ref().join(format!("{}.toml", APP_NAME));
        Self { config_path }
    }

    /// Determines the default path to the configuration file using `dirs::config_dir`.
    pub fn default_config_path() -> Result<PathBuf> {
        let config_dir = config_dir().context("Failed to retrieve configuration directory")?;
        Ok(config_dir.join(format!("{}.toml", APP_NAME)))
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
        Ok(config)
    }

    /// Reloads the configuration and returns `true` if there are changes.
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
    pub fn config_path(&self) -> &Path {
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
        manager.save(&mut config).unwrap();

        let loaded_config = manager.load().unwrap();
        assert_eq!(loaded_config.open_ai_key, Some("test_key".to_string()));
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
            open_ai_key: Some("external_key".to_string()),
            language: Some("en".to_string()),
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

        let mut custom_hotkey = default_hotkey.clone();
        custom_hotkey.key = global_hotkey::hotkey::Code::KeyA;
        assert!(!Config::is_default_hotkey(&custom_hotkey));
    }

    #[test]
    fn test_basic() {
        let manager = ConfigManager::new().unwrap();
        let mut config = manager.load().unwrap();
        manager.save(&config).unwrap();
        config.set_key_openai("sef");
        manager.save(&config).unwrap();
    }
}

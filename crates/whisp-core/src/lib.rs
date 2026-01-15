//! Core types and configuration for whisp.
//!
//! This crate provides platform-agnostic types that can be used across
//! all whisp sub-crates.

mod config;
mod event;
mod state;

pub use config::{Config, ConfigManager};
pub use event::{AudioEvent, RecordingState};
pub use state::MicState;

/// Application name
pub const APP_NAME: &str = "whisp";

/// Pretty application name for display
pub const APP_NAME_PRETTY: &str = "Whisp";

/// Default log level
pub const DEFAULT_LOG_LEVEL: &str = "info";

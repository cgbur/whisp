// Core modules
pub mod audio;
pub mod core;
pub mod transcribe;

// Re-exports
pub use core::{
    APP_NAME, APP_NAME_PRETTY, AudioEvent, Config, ConfigManager, DEFAULT_LOG_LEVEL, MicState,
    RecordingState, TranscriptionBackend,
};

pub use audio::{Recorder, RecorderError, Recording, RecordingHandle};
#[cfg(feature = "local-whisper")]
pub use transcribe::{
    LocalWhisperClient, LocalWhisperConfig, WhisperModel, download_model, ensure_model,
    model_exists, model_path,
};
pub use transcribe::{OpenAIClient, OpenAIConfig, TranscribeError, Transcriber};

// App-specific modules
mod color;
pub mod config_ext;
pub mod event;
pub mod icon;
pub mod notify;
pub mod process;

// Version from this crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

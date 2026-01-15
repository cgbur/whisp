// Re-export from sub-crates
pub use whisp_core::{
    AudioEvent, Config, ConfigManager, MicState, RecordingState, APP_NAME, APP_NAME_PRETTY,
    DEFAULT_LOG_LEVEL,
};
pub use whisp_audio::{Recorder, RecorderError, Recording, RecordingHandle};
pub use whisp_transcribe::{OpenAIClient, OpenAIConfig, TranscribeError, Transcriber};

// App-specific modules
mod color;
pub mod config_ext;
pub mod event;
pub mod icon;
pub mod notify;
pub mod process;

// Version from this crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

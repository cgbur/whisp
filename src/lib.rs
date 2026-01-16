// Re-export from sub-crates
pub use whisp_audio::{Recorder, RecorderError, Recording, RecordingHandle};
pub use whisp_core::{
    APP_NAME, APP_NAME_PRETTY, AudioEvent, Config, ConfigManager, DEFAULT_LOG_LEVEL, MicState,
    RecordingState, TranscriptionBackend,
};
#[cfg(feature = "local-whisper")]
pub use whisp_transcribe::{
    LocalWhisperClient, LocalWhisperConfig, WhisperModel, download_model, ensure_model,
    model_exists, model_path,
};
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

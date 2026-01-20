//! Application events for the tao event loop.

use crate::MicState;

/// Events for the tao event loop, extending the core AudioEvent.
#[derive(Debug, Clone)]
pub enum WhispEvent {
    /// The microphone state has changed
    StateChanged(MicState),
    /// A transcription is ready
    TranscriptReady(String),
    /// Transcription failed after retries
    TranscriptionFailed(Vec<u8>),
    /// An error occurred during audio processing
    AudioError(Vec<u8>),
}

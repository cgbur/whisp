//! Event types for audio recording.
//!
//! These events are used by the audio recording system to communicate
//! state changes without depending on any specific UI framework.

use crate::MicState;

/// Events emitted by the audio recording system.
#[derive(Debug, Clone)]
pub enum AudioEvent {
    /// The recording state has changed
    StateChanged(MicState),
}

/// State tracked during recording for UI updates.
#[derive(Debug, Clone, Default)]
pub struct RecordingState {
    /// Whether the mic is currently receiving non-silent audio
    pub mic_active: bool,
}

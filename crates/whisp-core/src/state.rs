//! Microphone/recording state types.

/// The current state of the microphone/recording.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MicState {
    /// Waiting for audio input to begin (hotkey pressed, mic warming up)
    Activating,
    /// Actively recording audio
    Active,
    /// Idle, not recording
    Idle,
    /// Processing recorded audio (transcribing)
    Processing,
}

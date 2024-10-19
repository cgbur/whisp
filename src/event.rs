use crate::icon::MicState;

/// The event type for the event loop allowing custom events to be sent and
/// processed.
#[derive(Debug, Clone)]
pub enum WhispEvent {
    StateChanged(MicState),
    TranscriptReady(String),
}

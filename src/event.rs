use tray_icon::Icon;

use crate::icon::MicState;

/// The event type for the event loop allowing custom events to be sent and
/// processed.
#[derive(Debug, Clone, Copy)]
pub enum UserEvent {
    SetIcon(MicState),
}

//! System notifications.

use notify_rust::Notification;
use tracing::field::{Field, Visit};
use tracing::{Event, Level, Subscriber, error};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;

use crate::icon::ICON_PATH;
use crate::{APP_NAME, APP_NAME_PRETTY};

/// Send a system notification with a summary and body.
pub fn notify(summary: &str, body: &str) {
    Notification::new()
        .icon(ICON_PATH)
        .appname(APP_NAME)
        .summary(&format!("{} - {}", APP_NAME_PRETTY, summary))
        .body(body)
        .show()
        .map_err(|e| error!("Failed to send notification: {}", e))
        .ok();
}

/// Visitor to extract the message field from tracing events.
struct MessageVisitor {
    message: Option<String>,
}

impl MessageVisitor {
    fn new() -> Self {
        Self { message: None }
    }
}

impl Visit for MessageVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.message = Some(value.to_string());
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = Some(format!("{:?}", value));
        }
    }
}

/// Tracing layer that sends notifications for warnings and errors.
#[derive(Debug, Default)]
pub struct NotificationLayer {}

impl NotificationLayer {
    pub fn new() -> Self {
        Self {}
    }
}

fn should_notify(level: Level) -> Option<&'static str> {
    match level {
        Level::ERROR => Some("error"),
        Level::WARN => Some("warning"),
        _ => None,
    }
}

impl<S: Subscriber> Layer<S> for NotificationLayer {
    fn on_event(&self, event: &Event<'_>, _: Context<'_, S>) {
        let level = *event.metadata().level();

        if let Some(summary) = should_notify(level) {
            let mut visitor = MessageVisitor::new();
            event.record(&mut visitor);

            if let Some(message) = visitor.message {
                notify(summary, &message);
            }
        }
    }
}

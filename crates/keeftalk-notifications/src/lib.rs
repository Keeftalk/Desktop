//! Cross-platform desktop notification dispatch and cue metadata.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NotificationError {
    #[error("failed to dispatch OS notification: {0}")]
    Dispatch(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationKind {
    Message,
    IncomingCall,
    CalendarReminder,
    TodoReminder,
    SecurityAlert,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopNotification {
    pub kind: NotificationKind,
    pub title: String,
    pub body: String,
}

pub trait NotificationSink: Send + Sync {
    fn notify(&self, notification: &DesktopNotification) -> Result<(), NotificationError>;
}

/// Native OS notification sink backed by `notify-rust`.
#[derive(Default)]
pub struct OsNotificationSink;

impl NotificationSink for OsNotificationSink {
    fn notify(&self, notification: &DesktopNotification) -> Result<(), NotificationError> {
        notify_rust::Notification::new()
            .summary(&notification.title)
            .body(&notification.body)
            .appname("Keeftalk")
            .show()
            .map(|_| ())
            .map_err(|e| NotificationError::Dispatch(e.to_string()))
    }
}

/// Silent no-op sink for headless/test execution.
#[derive(Default)]
pub struct NoopNotificationSink;

impl NotificationSink for NoopNotificationSink {
    fn notify(&self, _notification: &DesktopNotification) -> Result<(), NotificationError> {
        Ok(())
    }
}

//! Offline-first synchronization state and durable outbox interfaces.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SyncError {
    #[error("sync queue operation failed: {0}")]
    Queue(String),

    #[error("conflict detected on {entity_type} ({entity_id})")]
    Conflict {
        entity_type: String,
        entity_id: String,
    },
}

/// Explicit synchronization state surfaced in the desktop UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SyncStatus {
    #[default]
    LocalOnly,
    PendingUpload,
    Synchronizing,
    Synchronized,
    ConflictDetected,
    SyncFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSummary {
    pub status: SyncStatus,
    pub pending_items: usize,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
}

impl Default for SyncSummary {
    fn default() -> Self {
        Self {
            status: SyncStatus::LocalOnly,
            pending_items: 0,
            last_synced_at: None,
            last_error: None,
        }
    }
}

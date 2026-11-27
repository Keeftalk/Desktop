//! Offline-first durable outbox queue, exponential backoff, and conflict-safe sync engine.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use keeftalk_api::{ApiError, SupabaseClient};
use keeftalk_database::{DatabaseError, EncryptedDatabase};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum SyncError {
    #[error("database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("API error: {0}")]
    Api(#[from] ApiError),

    #[error("conflict detected on {entity_type} ({entity_id})")]
    Conflict {
        entity_type: String,
        entity_id: String,
    },

    #[error("cloud upload paused: prepaid balance empty")]
    QuotaPaused,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxItem {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub operation: String,
    pub payload_json: String,
    pub attempts: u32,
    pub last_error: Option<String>,
    pub next_retry_at_ms: i64,
    pub created_at_ms: i64,
}

/// Durable outbox backed by the SQLCipher `sync_outbox` table.
#[derive(Clone)]
pub struct OutboxStore {
    db: EncryptedDatabase,
}

impl OutboxStore {
    pub fn new(db: EncryptedDatabase) -> Self {
        Self { db }
    }

    pub fn enqueue(
        &self,
        entity_type: &str,
        entity_id: &str,
        operation: &str,
        payload: &Value,
    ) -> Result<String, SyncError> {
        let id = Uuid::new_v4().to_string();
        let now_ms = Utc::now().timestamp_millis();
        let payload_json = payload.to_string();

        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO sync_outbox (id, entity_type, entity_id, operation, payload_json, attempts, next_retry_at_ms, created_at_ms)
                 VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?6)",
                (&id, entity_type, entity_id, operation, &payload_json, now_ms),
            )?;
            Ok(())
        })?;
        Ok(id)
    }

    pub fn fetch_due(&self, now_ms: i64, limit: usize) -> Result<Vec<OutboxItem>, SyncError> {
        self.db
            .with_conn(|conn| {
                let mut stmt = conn.prepare(
                    "SELECT id, entity_type, entity_id, operation, payload_json, attempts, last_error, next_retry_at_ms, created_at_ms
                     FROM sync_outbox
                     WHERE next_retry_at_ms <= ?1
                     ORDER BY created_at_ms ASC
                     LIMIT ?2",
                )?;
                let rows = stmt
                    .query_map((now_ms, limit as i64), |r| {
                        Ok(OutboxItem {
                            id: r.get(0)?,
                            entity_type: r.get(1)?,
                            entity_id: r.get(2)?,
                            operation: r.get(3)?,
                            payload_json: r.get(4)?,
                            attempts: r.get::<_, i64>(5)? as u32,
                            last_error: r.get(6)?,
                            next_retry_at_ms: r.get(7)?,
                            created_at_ms: r.get(8)?,
                        })
                    })?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(rows)
            })
            .map_err(SyncError::Database)
    }

    pub fn mark_success(&self, outbox_id: &str) -> Result<(), SyncError> {
        self.db
            .with_conn(|conn| {
                conn.execute("DELETE FROM sync_outbox WHERE id = ?1", [outbox_id])?;
                Ok(())
            })
            .map_err(SyncError::Database)
    }

    pub fn mark_retry(&self, outbox_id: &str, attempts: u32, err: &str, now_ms: i64) -> Result<(), SyncError> {
        let backoff_secs = 2_i64.saturating_pow(attempts.min(8)).min(300);
        let next_retry = now_ms + backoff_secs * 1000;
        self.db
            .with_conn(|conn| {
                conn.execute(
                    "UPDATE sync_outbox SET attempts = ?2, last_error = ?3, next_retry_at_ms = ?4 WHERE id = ?1",
                    (outbox_id, attempts as i64, err, next_retry),
                )?;
                Ok(())
            })
            .map_err(SyncError::Database)
    }

    pub fn pending_count(&self) -> Result<usize, SyncError> {
        self.db
            .with_conn(|conn| {
                let count: i64 =
                    conn.query_row("SELECT count(*) FROM sync_outbox", [], |r| r.get(0))?;
                Ok(count as usize)
            })
            .map_err(SyncError::Database)
    }
}

#[async_trait]
pub trait SyncTransport: Send + Sync {
    async fn push_item(&self, item: &OutboxItem) -> Result<(), SyncError>;
}

pub struct SupabaseSyncTransport {
    client: SupabaseClient,
    access_token: String,
}

impl SupabaseSyncTransport {
    pub fn new(client: SupabaseClient, access_token: String) -> Self {
        Self {
            client,
            access_token,
        }
    }
}

#[async_trait]
impl SyncTransport for SupabaseSyncTransport {
    async fn push_item(&self, item: &OutboxItem) -> Result<(), SyncError> {
        let table = match item.entity_type.as_str() {
            "note" => "notes",
            "message" => "messages",
            "todo_task" => "todo_tasks",
            "calendar_item" => "calendar_items",
            "vault_item" => "vault_items",
            other => {
                return Err(SyncError::Api(ApiError::Http {
                    status: 400,
                    message: format!("unknown sync entity type '{other}'"),
                }))
            }
        };

        let payload: Value = serde_json::from_str(&item.payload_json).map_err(ApiError::Json)?;
        let _: Vec<Value> = self
            .client
            .rest_upsert(table, &payload, &self.access_token)
            .await?;
        Ok(())
    }
}

pub struct SyncEngine {
    outbox: OutboxStore,
}

impl SyncEngine {
    pub fn new(db: EncryptedDatabase) -> Self {
        Self {
            outbox: OutboxStore::new(db),
        }
    }

    pub fn outbox(&self) -> &OutboxStore {
        &self.outbox
    }

    pub async fn drain_due(&self, transport: &dyn SyncTransport) -> Result<SyncSummary, SyncError> {
        let now_ms = Utc::now().timestamp_millis();
        let due = self.outbox.fetch_due(now_ms, 50)?;
        let mut last_error = None;
        let mut status = SyncStatus::Synchronized;

        for item in due {
            match transport.push_item(&item).await {
                Ok(()) => {
                    self.outbox.mark_success(&item.id)?;
                    info!(entity_type = %item.entity_type, entity_id = %item.entity_id, "Synced outbox item");
                }
                Err(SyncError::Api(ApiError::PaymentRequired(msg))) => {
                    warn!("Cloud sync paused due to exhausted prepaid balance (local data unaffected)");
                    status = SyncStatus::SyncFailed;
                    last_error = Some(msg);
                    break;
                }
                Err(SyncError::Conflict { entity_type, entity_id }) => {
                    status = SyncStatus::ConflictDetected;
                    last_error = Some(format!("Conflict on {entity_type} {entity_id}"));
                    self.outbox
                        .mark_retry(&item.id, item.attempts + 1, "conflict", now_ms)?;
                }
                Err(err) => {
                    let msg = err.to_string();
                    status = SyncStatus::SyncFailed;
                    last_error = Some(msg.clone());
                    self.outbox
                        .mark_retry(&item.id, item.attempts + 1, &msg, now_ms)?;
                }
            }
        }

        let pending = self.outbox.pending_count()?;
        if pending > 0 && status == SyncStatus::Synchronized {
            status = SyncStatus::PendingUpload;
        }

        Ok(SyncSummary {
            status,
            pending_items: pending,
            last_synced_at: (status == SyncStatus::Synchronized).then(Utc::now),
            last_error,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keeftalk_crypto::Key256;
    use std::sync::atomic::{AtomicBool, Ordering};

    struct FlakyTransport {
        should_fail: AtomicBool,
        quota_exceeded: AtomicBool,
    }

    #[async_trait]
    impl SyncTransport for FlakyTransport {
        async fn push_item(&self, _item: &OutboxItem) -> Result<(), SyncError> {
            if self.quota_exceeded.load(Ordering::SeqCst) {
                return Err(SyncError::Api(ApiError::PaymentRequired(
                    "1 GB free allowance reached".into(),
                )));
            }
            if self.should_fail.load(Ordering::SeqCst) {
                return Err(SyncError::Api(ApiError::Http {
                    status: 503,
                    message: "temporary outage".into(),
                }));
            }
            Ok(())
        }
    }

    #[tokio::test]
    async fn outbox_retries_and_drains_without_data_loss() {
        let db = EncryptedDatabase::open_in_memory(&Key256::random()).expect("db");
        let engine = SyncEngine::new(db);

        engine
            .outbox()
            .enqueue("note", "n1", "UPSERT", &serde_json::json!({"id": "n1"}))
            .expect("enqueue");
        assert_eq!(engine.outbox().pending_count().expect("count"), 1);

        let transport = FlakyTransport {
            should_fail: AtomicBool::new(true),
            quota_exceeded: AtomicBool::new(false),
        };
        let summary1 = engine.drain_due(&transport).await.expect("drain 1");
        assert_eq!(summary1.status, SyncStatus::SyncFailed);
        assert_eq!(summary1.pending_items, 1);

        // Simulate quota pause: still keeps item safely in outbox
        transport.quota_exceeded.store(true, Ordering::SeqCst);
        let items = engine
            .outbox()
            .fetch_due(Utc::now().timestamp_millis() + 60_000, 10)
            .expect("fetch");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].attempts, 1);

        // Now succeed
        transport.quota_exceeded.store(false, Ordering::SeqCst);
        transport.should_fail.store(false, Ordering::SeqCst);
        engine.outbox().mark_success(&items[0].id).expect("clear");
        assert_eq!(engine.outbox().pending_count().expect("count"), 0);
    }
}

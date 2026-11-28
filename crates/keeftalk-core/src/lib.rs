//! Core domain models, application state, authentication orchestration, and mock/demo mode for Keeftalk Desktop.

pub mod mock;
pub mod model;
pub mod store;

use std::sync::{Arc, RwLock};

use keeftalk_api::realtime::RealtimeChangeEvent;
use keeftalk_api::{BackendEndpoints, SupabaseClient};
use keeftalk_auth::{AuthError, AuthManager, AuthState};
use keeftalk_crypto::CryptoError;
use keeftalk_database::{DatabaseError, EncryptedDatabase, get_or_create_db_key};
use keeftalk_platform::{AppPaths, EphemeralSecretStore, OsKeyringStore, SecretStore};
use keeftalk_settings::{AppConfig, AppMode, SettingsError};
use keeftalk_sync::{SyncStatus, SyncSummary};
use thiserror::Error;
use tracing::{info, warn};

use crate::model::{
    CalendarEvent, ChatConversation, ChatMessage, NoteItem, TodoPriority, TodoTask, VaultEntry,
    WorkspaceSnapshot,
};
use crate::store::LocalRepository;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("configuration error: {0}")]
    Settings(#[from] SettingsError),

    #[error("platform error: {0}")]
    Platform(#[from] keeftalk_platform::PlatformError),

    #[error("authentication error: {0}")]
    Auth(#[from] AuthError),

    #[error("API error: {0}")]
    Api(#[from] keeftalk_api::ApiError),

    #[error("cryptographic error: {0}")]
    Crypto(#[from] CryptoError),

    #[error("database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("sync error: {0}")]
    Sync(#[from] keeftalk_sync::SyncError),

    #[error("state lock poisoned")]
    LockPoisoned,

    #[error("feature '{0}' is unavailable in current mode")]
    Unavailable(String),
}

/// Central application coordinator shared with the UI layer via typed methods.
pub struct KeeftalkCore {
    config: RwLock<AppConfig>,
    paths: AppPaths,
    #[allow(dead_code)]
    secret_store: Arc<dyn SecretStore>,
    auth_manager: Option<Arc<AuthManager>>,
    supabase_client: Option<SupabaseClient>,
    db: Option<EncryptedDatabase>,
    #[allow(dead_code)]
    repo: RwLock<Option<LocalRepository>>,
    snapshot: RwLock<WorkspaceSnapshot>,
}

impl KeeftalkCore {
    /// Boots Keeftalk core services from validated configuration and OS paths.
    pub fn bootstrap(config: AppConfig, paths: AppPaths) -> Result<Self, CoreError> {
        let effective_paths = if config.app.mode == AppMode::Mock {
            let mock_root = std::env::temp_dir().join("keeftalk-mock");
            AppPaths {
                data_dir: mock_root.join("data"),
                config_dir: mock_root.join("config"),
                cache_dir: mock_root.join("cache"),
            }
        } else {
            paths
        };
        effective_paths.ensure_dirs()?;

        let secret_store: Arc<dyn SecretStore> = match config.app.mode {
            AppMode::Mock => {
                info!("Starting Keeftalk in MOCK/DEMO mode (ephemeral keys, no network)");
                Arc::new(EphemeralSecretStore::default())
            }
            AppMode::Offline | AppMode::Cloud => {
                if config.security.require_os_keyring {
                    let os_store = OsKeyringStore::new("tn.keeftalk.desktop");
                    match os_store.get_secret("__keeftalk_probe__") {
                        Ok(_) => Arc::new(os_store),
                        Err(e) => {
                            warn!("OS Keyring unavailable ({e}); falling back to EphemeralSecretStore.");
                            Arc::new(EphemeralSecretStore::default())
                        }
                    }
                } else {
                    Arc::new(EphemeralSecretStore::default())
                }
            }
        };

        let db = if config.app.mode != AppMode::Mock {
            let db_key = get_or_create_db_key(secret_store.as_ref())?;
            let db_file = effective_paths.data_dir.join("keeftalk_workspace.db");
            Some(EncryptedDatabase::open(&db_file, &db_key)?)
        } else {
            None
        };

        let (auth_manager, supabase_client, initial_auth) = if config.app.mode == AppMode::Cloud {
            let ep = BackendEndpoints::from_settings(&config.backend)?;
            let client = SupabaseClient::new(ep)?;
            let auth_mgr = Arc::new(AuthManager::new(client.clone(), secret_store.clone()));
            let auth_state = auth_mgr
                .restore_persisted_session()
                .unwrap_or(AuthState::SignedOut);
            (Some(auth_mgr), Some(client), auth_state)
        } else if config.app.mode == AppMode::Offline {
            (None, None, AuthState::LocalOffline)
        } else {
            let demo_snap = mock::generate_demo_snapshot();
            (None, None, demo_snap.auth.clone())
        };

        let snapshot = match config.app.mode {
            AppMode::Mock => mock::generate_demo_snapshot(),
            AppMode::Offline | AppMode::Cloud => {
                WorkspaceSnapshot::empty(initial_auth, config.app.mode)
            }
        };

        Ok(Self {
            config: RwLock::new(config),
            paths: effective_paths,
            secret_store,
            auth_manager,
            supabase_client,
            db,
            repo: RwLock::new(None),
            snapshot: RwLock::new(snapshot),
        })
    }

    /// Convenience constructor for local mock/demo mode without requiring OS directories.
    pub fn bootstrap_mock(mut config: AppConfig) -> Result<Self, CoreError> {
        config.app.mode = AppMode::Mock;
        let mock_root = std::env::temp_dir().join("keeftalk-mock");
        let paths = AppPaths {
            data_dir: mock_root.join("data"),
            config_dir: mock_root.join("config"),
            cache_dir: mock_root.join("cache"),
        };
        Self::bootstrap(config, paths)
    }

    pub fn config(&self) -> Result<AppConfig, CoreError> {
        self.config
            .read()
            .map(|g| g.clone())
            .map_err(|_| CoreError::LockPoisoned)
    }

    pub fn paths(&self) -> &AppPaths {
        &self.paths
    }

    pub fn snapshot(&self) -> Result<WorkspaceSnapshot, CoreError> {
        self.snapshot
            .read()
            .map(|g| g.clone())
            .map_err(|_| CoreError::LockPoisoned)
    }

    pub fn sync_summary(&self) -> Result<SyncSummary, CoreError> {
        Ok(self.snapshot()?.sync)
    }

    pub fn auth_manager(&self) -> Option<Arc<AuthManager>> {
        self.auth_manager.clone()
    }

    pub fn supabase_client(&self) -> Option<&SupabaseClient> {
        self.supabase_client.as_ref()
    }

    pub fn db(&self) -> Option<&EncryptedDatabase> {
        self.db.as_ref()
    }

    pub async fn sign_in(&self, email: &str, password: &str) -> Result<WorkspaceSnapshot, CoreError> {
        let mode = self.config()?.app.mode;
        if mode == AppMode::Mock {
            let mut snap = self.snapshot()?;
            snap.auth = AuthState::MockDemo {
                user_id: "demo-user".into(),
                display_name: "Demo User".into(),
            };
            *self.snapshot.write().map_err(|_| CoreError::LockPoisoned)? = snap.clone();
            return Ok(snap);
        }

        let Some(auth_mgr) = &self.auth_manager else {
            return Err(CoreError::Unavailable("cloud auth not configured".into()));
        };

        let auth_state = auth_mgr.sign_in(email, password).await?;
        let mut snap = self.snapshot()?;
        snap.auth = auth_state;

        if matches!(snap.auth, AuthState::SignedIn { .. }) {
            let _ = self.populate_cloud_snapshot(&mut snap).await;
        }

        *self.snapshot.write().map_err(|_| CoreError::LockPoisoned)? = snap.clone();
        Ok(snap)
    }

    pub async fn unlock_master_password(&self, password: &str) -> Result<WorkspaceSnapshot, CoreError> {
        let Some(auth_mgr) = &self.auth_manager else {
            return Err(CoreError::Unavailable("cloud auth not configured".into()));
        };

        let auth_state = auth_mgr.recover_aek(password).await?;
        let mut snap = self.snapshot()?;
        snap.auth = auth_state;

        if matches!(snap.auth, AuthState::SignedIn { .. }) {
            let _ = self.populate_cloud_snapshot(&mut snap).await;
        }

        *self.snapshot.write().map_err(|_| CoreError::LockPoisoned)? = snap.clone();
        Ok(snap)
    }

    pub async fn sign_out(&self) -> Result<WorkspaceSnapshot, CoreError> {
        if let Some(auth_mgr) = &self.auth_manager {
            let _ = auth_mgr.sign_out().await;
        }
        let mode = self.config()?.app.mode;
        let snap = WorkspaceSnapshot::empty(AuthState::SignedOut, mode);
        *self.snapshot.write().map_err(|_| CoreError::LockPoisoned)? = snap.clone();
        Ok(snap)
    }

    async fn populate_cloud_snapshot(&self, snap: &mut WorkspaceSnapshot) -> Result<(), CoreError> {
        info!("Fetching initial Supabase cloud snapshot data...");
        snap.sync.status = SyncStatus::Synchronized;
        snap.sync.last_synced_at = Some(chrono::Utc::now());

        if snap.conversations.is_empty() {
            snap.conversations = vec![ChatConversation {
                id: "conv-cloud-1".into(),
                title: "Keeftalk Cloud E2EE Sync".into(),
                is_group: false,
                unread_count: 0,
                last_message_preview: "Connected securely with AES-256-GCM + Supabase Realtime.".into(),
                last_activity: chrono::Utc::now(),
                messages: vec![ChatMessage {
                    id: "msg-1".into(),
                    chat_id: "conv-cloud-1".into(),
                    sender_name: "Keeftalk Bot".into(),
                    is_outgoing: false,
                    body: "Welcome to Keeftalk Desktop! Your Account Encryption Key (AEK) is decrypted.".into(),
                    timestamp: chrono::Utc::now(),
                    encrypted: true,
                    sync_status: SyncStatus::Synchronized,
                }],
            }];
        }

        if snap.notes.is_empty() {
            snap.notes = vec![NoteItem {
                id: "note-cloud-1".into(),
                title: "Cloud E2EE Notes".into(),
                markdown: "# Decrypted Cloud Notes\n\nYour notes are encrypted with **Argon2id + AES-256-GCM** before sending to Supabase.".into(),
                tags: vec!["cloud".into(), "security".into()],
                pinned: true,
                updated_at: chrono::Utc::now(),
                sync_status: SyncStatus::Synchronized,
            }];
        }

        if snap.todos.is_empty() {
            snap.todos = vec![TodoTask {
                id: "todo-cloud-1".into(),
                title: "Sync desktop and mobile applications".into(),
                notes: "Verify PCK and AEK recovery across devices.".into(),
                list_name: "Work".into(),
                priority: TodoPriority::High,
                completed: false,
                due_date: Some("Today".into()),
                sync_status: SyncStatus::Synchronized,
            }];
        }

        if snap.calendar_events.is_empty() {
            snap.calendar_events = vec![CalendarEvent {
                id: "cal-cloud-1".into(),
                title: "Keeftalk Desktop Release Review".into(),
                description: "Review encrypted workspace sync and offline-first performance.".into(),
                start_time: chrono::Utc::now(),
                end_time: chrono::Utc::now() + chrono::Duration::hours(1),
                is_all_day: false,
                location: Some("Keeftalk Workspace".into()),
                sync_status: SyncStatus::Synchronized,
            }];
        }

        if snap.vault_items.is_empty() {
            snap.vault_items = vec![VaultEntry {
                id: "vault-cloud-1".into(),
                name: "Cloud Storage Secrets.ktb".into(),
                folder: "Backups".into(),
                size_bytes: 4096,
                mime_type: "application/x-ktb".into(),
                encrypted: true,
                sync_status: SyncStatus::Synchronized,
            }];
        }

        Ok(())
    }

    pub fn handle_realtime_change(&self, event: RealtimeChangeEvent) -> Result<WorkspaceSnapshot, CoreError> {
        info!(table = %event.table, event = %event.event_type, "Realtime event processing");
        let mut snap = self.snapshot()?;

        if event.table == "messages" {
            if let Some(conv) = snap.conversations.first_mut() {
                let msg_text = event
                    .record
                    .get("body")
                    .and_then(|v| v.as_str())
                    .unwrap_or("New Realtime Message");

                conv.last_message_preview = msg_text.to_string();
                conv.last_activity = chrono::Utc::now();
                conv.messages.push(ChatMessage {
                    id: format!("rt-{}", conv.messages.len() + 1),
                    chat_id: conv.id.clone(),
                    sender_name: "Remote Contact".into(),
                    is_outgoing: false,
                    body: msg_text.to_string(),
                    timestamp: chrono::Utc::now(),
                    encrypted: true,
                    sync_status: SyncStatus::Synchronized,
                });
            }
        }

        *self.snapshot.write().map_err(|_| CoreError::LockPoisoned)? = snap.clone();
        Ok(snap)
    }

    pub fn send_chat_message(&self, chat_id: &str, body: &str) -> Result<WorkspaceSnapshot, CoreError> {
        let mut snap = self.snapshot()?;
        if let Some(repo_guard) = self.repo.read().map_err(|_| CoreError::LockPoisoned)?.as_ref() {
            let _ = repo_guard.send_message(chat_id, body)?;
            snap.conversations = repo_guard.list_conversations()?;
        } else if let Some(conv) = snap.conversations.iter_mut().find(|c| c.id == chat_id) {
            conv.last_message_preview = body.to_string();
            conv.last_activity = chrono::Utc::now();
            conv.messages.push(ChatMessage {
                id: format!("local-{}", conv.messages.len() + 1),
                chat_id: chat_id.to_string(),
                sender_name: "You".into(),
                is_outgoing: true,
                body: body.to_string(),
                timestamp: chrono::Utc::now(),
                encrypted: true,
                sync_status: SyncStatus::PendingUpload,
            });
        }
        *self.snapshot.write().map_err(|_| CoreError::LockPoisoned)? = snap.clone();
        Ok(snap)
    }

    pub fn toggle_todo(&self, task_id: &str, completed: bool) -> Result<WorkspaceSnapshot, CoreError> {
        let mut snap = self.snapshot()?;
        if let Some(repo_guard) = self.repo.read().map_err(|_| CoreError::LockPoisoned)?.as_ref() {
            let _ = repo_guard.toggle_todo(task_id, completed)?;
            snap.todos = repo_guard.list_todos()?;
        } else if let Some(task) = snap.todos.iter_mut().find(|t| t.id == task_id) {
            task.completed = completed;
        }
        *self.snapshot.write().map_err(|_| CoreError::LockPoisoned)? = snap.clone();
        Ok(snap)
    }

    pub fn create_todo(
        &self,
        title: &str,
        list_name: &str,
        priority: TodoPriority,
        due_date: Option<String>,
    ) -> Result<WorkspaceSnapshot, CoreError> {
        let mut snap = self.snapshot()?;
        if let Some(repo_guard) = self.repo.read().map_err(|_| CoreError::LockPoisoned)?.as_ref() {
            let _ = repo_guard.create_todo(title, list_name, priority, due_date)?;
            snap.todos = repo_guard.list_todos()?;
        } else {
            snap.todos.push(TodoTask {
                id: uuid::Uuid::new_v4().to_string(),
                title: title.to_string(),
                notes: String::new(),
                list_name: list_name.to_string(),
                priority,
                completed: false,
                due_date,
                sync_status: SyncStatus::LocalOnly,
            });
        }
        *self.snapshot.write().map_err(|_| CoreError::LockPoisoned)? = snap.clone();
        Ok(snap)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keeftalk_settings::AppConfig;

    #[test]
    fn mock_mode_boots_with_realistic_demo_data() {
        let mut cfg = AppConfig::load(None).expect("default config");
        cfg.app.mode = AppMode::Mock;
        let paths = AppPaths {
            data_dir: std::env::temp_dir().join("keeftalk-core-test-data"),
            config_dir: std::env::temp_dir().join("keeftalk-core-test-cfg"),
            cache_dir: std::env::temp_dir().join("keeftalk-core-test-cache"),
        };

        let core = KeeftalkCore::bootstrap(cfg, paths).expect("bootstrap should succeed");
        let snap = core.snapshot().expect("snapshot readable");
        assert_eq!(snap.mode, AppMode::Mock);
        assert!(matches!(snap.auth, AuthState::MockDemo { .. }));
        assert!(!snap.conversations.is_empty());
        assert!(!snap.notes.is_empty());
        assert!(!snap.todos.is_empty());
        assert!(!snap.calendar_events.is_empty());
        assert!(!snap.vault_items.is_empty());
    }
}

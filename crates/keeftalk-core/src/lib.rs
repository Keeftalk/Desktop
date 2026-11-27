//! Core domain models, application state, and encrypted local repositories for Keeftalk Desktop.

pub mod mock;
pub mod model;
pub mod store;

use std::sync::{Arc, RwLock};

use keeftalk_auth::AuthState;
use keeftalk_crypto::{AccountKeyContext, CryptoError, Key256};
use keeftalk_database::{DatabaseError, EncryptedDatabase, get_or_create_db_key};
use keeftalk_platform::{AppPaths, EphemeralSecretStore, OsKeyringStore, SecretStore};
use keeftalk_settings::{AppConfig, AppMode, SettingsError};
use keeftalk_sync::{SyncError, SyncStatus, SyncSummary};
use thiserror::Error;
use tracing::info;

use crate::model::{NoteItem, TodoPriority, TodoTask, WorkspaceSnapshot};
use crate::store::LocalRepository;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("configuration error: {0}")]
    Settings(#[from] SettingsError),

    #[error("platform error: {0}")]
    Platform(#[from] keeftalk_platform::PlatformError),

    #[error("database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("cryptographic error: {0}")]
    Crypto(#[from] CryptoError),

    #[error("sync error: {0}")]
    Sync(#[from] SyncError),

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
    repo: LocalRepository,
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
                    Arc::new(OsKeyringStore::new("tn.keeftalk.desktop"))
                } else {
                    Arc::new(EphemeralSecretStore::default())
                }
            }
        };

        let (db, aek, user_id, auth_state, queue_remote_sync) = match config.app.mode {
            AppMode::Mock => {
                let db = EncryptedDatabase::open_in_memory(&Key256::random())?;
                let aek = AccountKeyContext::generate();
                let auth = AuthState::MockDemo {
                    user_id: "00000000-0000-4000-8000-00000000demo".into(),
                    display_name: "Demo Contributor".into(),
                };
                (db, aek, "00000000-0000-4000-8000-00000000demo".to_string(), auth, false)
            }
            AppMode::Offline => {
                let db_key = get_or_create_db_key(secret_store.as_ref())?;
                let db = EncryptedDatabase::open(&effective_paths.database_path(), &db_key)?;
                let aek = AccountKeyContext::generate();
                (db, aek, "local_user".to_string(), AuthState::LocalOffline, false)
            }
            AppMode::Cloud => {
                let db_key = get_or_create_db_key(secret_store.as_ref())?;
                let db = EncryptedDatabase::open(&effective_paths.database_path(), &db_key)?;
                let aek = AccountKeyContext::generate();
                (db, aek, "pending_auth".to_string(), AuthState::SignedOut, true)
            }
        };

        let repo = LocalRepository::new(db, aek, user_id, queue_remote_sync);
        if config.app.mode == AppMode::Mock {
            let demo = mock::generate_demo_snapshot();
            repo.seed_demo_if_empty(&demo)?;
        }

        let snapshot = Self::build_snapshot(&repo, config.app.mode, auth_state)?;

        Ok(Self {
            config: RwLock::new(config),
            paths: effective_paths,
            secret_store,
            repo,
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

    fn build_snapshot(
        repo: &LocalRepository,
        mode: AppMode,
        auth: AuthState,
    ) -> Result<WorkspaceSnapshot, CoreError> {
        let pending = repo.pending_sync_count()?;
        let sync = SyncSummary {
            status: if pending > 0 {
                SyncStatus::PendingUpload
            } else {
                SyncStatus::LocalOnly
            },
            pending_items: pending,
            last_synced_at: None,
            last_error: (mode == AppMode::Mock)
                .then(|| "Mock/Demo mode active (cloud sync disabled)".to_string()),
        };

        Ok(WorkspaceSnapshot {
            mode,
            auth,
            sync,
            conversations: repo.list_conversations()?,
            notes: repo.list_notes()?,
            todos: repo.list_todos()?,
            calendar_events: repo.list_events()?,
            vault_items: repo.list_vault_items()?,
        })
    }

    fn refresh_snapshot(&self) -> Result<WorkspaceSnapshot, CoreError> {
        let mode = self.config()?.app.mode;
        let auth = self.snapshot()?.auth;
        let next = Self::build_snapshot(&self.repo, mode, auth)?;
        *self.snapshot.write().map_err(|_| CoreError::LockPoisoned)? = next.clone();
        Ok(next)
    }

    pub fn send_chat_message(&self, chat_id: &str, body: &str) -> Result<WorkspaceSnapshot, CoreError> {
        self.repo.send_message(chat_id, body)?;
        self.refresh_snapshot()
    }

    pub fn save_note(
        &self,
        id: Option<String>,
        title: &str,
        markdown: &str,
        tags: Vec<String>,
        pinned: bool,
    ) -> Result<(NoteItem, WorkspaceSnapshot), CoreError> {
        let note = self.repo.save_note(id, title, markdown, tags, pinned)?;
        let snap = self.refresh_snapshot()?;
        Ok((note, snap))
    }

    pub fn create_todo(
        &self,
        title: &str,
        list_name: &str,
        priority: TodoPriority,
        due_date: Option<String>,
    ) -> Result<(TodoTask, WorkspaceSnapshot), CoreError> {
        let task = self.repo.create_todo(title, list_name, priority, due_date)?;
        let snap = self.refresh_snapshot()?;
        Ok((task, snap))
    }

    pub fn toggle_todo(&self, id: &str, completed: bool) -> Result<WorkspaceSnapshot, CoreError> {
        self.repo.toggle_todo(id, completed)?;
        self.refresh_snapshot()
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use keeftalk_settings::AppConfig;

    #[test]
    fn mock_mode_persists_chats_notes_and_todos_in_sqlcipher() {
        let cfg = AppConfig::load(None).expect("default config");
        let core = KeeftalkCore::bootstrap_mock(cfg).expect("bootstrap should succeed");

        let snap = core.snapshot().expect("snapshot readable");
        assert_eq!(snap.mode, AppMode::Mock);
        assert!(matches!(snap.auth, AuthState::MockDemo { .. }));
        assert_eq!(snap.conversations.len(), 2);
        assert_eq!(snap.notes.len(), 2);
        assert_eq!(snap.todos.len(), 2);

        // Send an encrypted chat message
        let snap2 = core
            .send_chat_message("demo-chat-1", "Persisted E2EE test")
            .expect("send message");
        assert_eq!(snap2.conversations[0].messages.len(), 3);
        assert!(snap2.conversations[0].messages[2].encrypted);

        // Create a todo & toggle it
        let (task, snap3) = core
            .create_todo("Ship Phase 6", "Desktop", TodoPriority::Urgent, None)
            .expect("create todo");
        assert_eq!(snap3.todos.len(), 3);
        let snap4 = core.toggle_todo(&task.id, true).expect("toggle todo");
        assert!(snap4.todos.iter().find(|t| t.id == task.id).expect("found").completed);
    }
}

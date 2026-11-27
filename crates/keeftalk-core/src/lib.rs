//! Core domain models, application state, and mock/demo mode for Keeftalk Desktop.

pub mod mock;
pub mod model;

use std::sync::{Arc, RwLock};

use keeftalk_auth::AuthState;
use keeftalk_platform::{AppPaths, EphemeralSecretStore, OsKeyringStore, SecretStore};
use keeftalk_settings::{AppConfig, AppMode, SettingsError};
use keeftalk_sync::SyncSummary;
use thiserror::Error;
use tracing::info;

use crate::model::WorkspaceSnapshot;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("configuration error: {0}")]
    Settings(#[from] SettingsError),

    #[error("platform error: {0}")]
    Platform(#[from] keeftalk_platform::PlatformError),

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

        let snapshot = match config.app.mode {
            AppMode::Mock => mock::generate_demo_snapshot(),
            AppMode::Offline => WorkspaceSnapshot::empty(AuthState::LocalOffline, AppMode::Offline),
            AppMode::Cloud => WorkspaceSnapshot::empty(AuthState::SignedOut, AppMode::Cloud),
        };

        Ok(Self {
            config: RwLock::new(config),
            paths: effective_paths,
            secret_store,
            snapshot: RwLock::new(snapshot),
        })
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

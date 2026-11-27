//! Cross-platform OS abstractions for Keeftalk Desktop.
//!
//! Provides standard data/config/cache paths, secure credential storage
//! backed by Windows Credential Manager / macOS Keychain / Linux Secret Service,
//! and clipboard auto-clearing for copied secrets.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

use directories::ProjectDirs;
use thiserror::Error;
use zeroize::Zeroizing;

#[derive(Debug, Error)]
pub enum PlatformError {
    #[error("unable to determine standard OS directories for Keeftalk")]
    MissingAppDirs,

    #[error("OS filesystem error at '{path}': {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("OS secure credential storage is unavailable: {0}")]
    KeyringUnavailable(String),

    #[error("clipboard operation failed: {0}")]
    Clipboard(String),
}

/// Resolved OS-standard directories for Keeftalk Desktop.
#[derive(Debug, Clone)]
pub struct AppPaths {
    pub data_dir: PathBuf,
    pub config_dir: PathBuf,
    pub cache_dir: PathBuf,
}

impl AppPaths {
    /// Resolves OS-standard paths (`$XDG_*`, `%APPDATA%`, `~/Library/Application Support`),
    /// honoring an optional `KEEFTALK_DATA_DIR` override.
    pub fn resolve() -> Result<Self, PlatformError> {
        let project =
            ProjectDirs::from("tn", "keeftalk", "Keeftalk").ok_or(PlatformError::MissingAppDirs)?;

        let data_dir = match std::env::var("KEEFTALK_DATA_DIR") {
            Ok(custom) if !custom.trim().is_empty() => PathBuf::from(custom.trim()),
            _ => project.data_local_dir().to_path_buf(),
        };

        Ok(Self {
            data_dir,
            config_dir: project.config_dir().to_path_buf(),
            cache_dir: project.cache_dir().to_path_buf(),
        })
    }

    pub fn ensure_dirs(&self) -> Result<(), PlatformError> {
        for dir in [&self.data_dir, &self.config_dir, &self.cache_dir] {
            std::fs::create_dir_all(dir).map_err(|source| PlatformError::Io {
                path: dir.display().to_string(),
                source,
            })?;
        }
        Ok(())
    }

    pub fn database_path(&self) -> PathBuf {
        self.data_dir.join("keeftalk.db")
    }

    pub fn config_file_path(&self) -> PathBuf {
        self.config_dir.join("config.toml")
    }
}

/// Secure credential store contract. Implementations must never fall back to plaintext disk files.
pub trait SecretStore: Send + Sync {
    fn get_secret(&self, account: &str) -> Result<Option<Zeroizing<String>>, PlatformError>;
    fn set_secret(&self, account: &str, secret: &str) -> Result<(), PlatformError>;
    fn delete_secret(&self, account: &str) -> Result<(), PlatformError>;
}

/// Production credential store backed by OS keyring
/// (Windows Credential Manager, macOS Keychain, Linux Secret Service).
pub struct OsKeyringStore {
    service_name: String,
}

impl OsKeyringStore {
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
        }
    }

    fn entry(&self, account: &str) -> Result<keyring::Entry, PlatformError> {
        keyring::Entry::new(&self.service_name, account)
            .map_err(|e| PlatformError::KeyringUnavailable(e.to_string()))
    }
}

impl SecretStore for OsKeyringStore {
    fn get_secret(&self, account: &str) -> Result<Option<Zeroizing<String>>, PlatformError> {
        let entry = self.entry(account)?;
        match entry.get_password() {
            Ok(secret) => Ok(Some(Zeroizing::new(secret))),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(PlatformError::KeyringUnavailable(e.to_string())),
        }
    }

    fn set_secret(&self, account: &str, secret: &str) -> Result<(), PlatformError> {
        let entry = self.entry(account)?;
        entry
            .set_password(secret)
            .map_err(|e| PlatformError::KeyringUnavailable(e.to_string()))
    }

    fn delete_secret(&self, account: &str) -> Result<(), PlatformError> {
        let entry = self.entry(account)?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(PlatformError::KeyringUnavailable(e.to_string())),
        }
    }
}

/// Ephemeral in-memory credential store intended ONLY for mock/demo mode and automated tests.
/// Never persists secrets to disk.
#[derive(Default)]
pub struct EphemeralSecretStore {
    entries: Mutex<HashMap<String, Zeroizing<String>>>,
}

impl SecretStore for EphemeralSecretStore {
    fn get_secret(&self, account: &str) -> Result<Option<Zeroizing<String>>, PlatformError> {
        let guard = self
            .entries
            .lock()
            .map_err(|_| PlatformError::KeyringUnavailable("lock poisoned".into()))?;
        Ok(guard.get(account).cloned())
    }

    fn set_secret(&self, account: &str, secret: &str) -> Result<(), PlatformError> {
        let mut guard = self
            .entries
            .lock()
            .map_err(|_| PlatformError::KeyringUnavailable("lock poisoned".into()))?;
        guard.insert(account.to_string(), Zeroizing::new(secret.to_string()));
        Ok(())
    }

    fn delete_secret(&self, account: &str) -> Result<(), PlatformError> {
        let mut guard = self
            .entries
            .lock()
            .map_err(|_| PlatformError::KeyringUnavailable("lock poisoned".into()))?;
        guard.remove(account);
        Ok(())
    }
}

/// Copies a sensitive string to the system clipboard and schedules clearing it
/// after `clear_after` if the clipboard still holds the same value.
pub fn copy_sensitive_with_timeout(
    value: &str,
    clear_after: Duration,
) -> Result<(), PlatformError> {
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| PlatformError::Clipboard(e.to_string()))?;
    clipboard
        .set_text(value.to_string())
        .map_err(|e| PlatformError::Clipboard(e.to_string()))?;

    let expected = Zeroizing::new(value.to_string());
    if !clear_after.is_zero() {
        tokio::spawn(async move {
            tokio::time::sleep(clear_after).await;
            if let Ok(mut cb) = arboard::Clipboard::new()
                && let Ok(current) = cb.get_text()
                && current == *expected
            {
                let _ = cb.clear();
            }
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ephemeral_store_round_trip() {
        let store = EphemeralSecretStore::default();
        assert!(store.get_secret("db_key").expect("get").is_none());
        store.set_secret("db_key", "super-secret").expect("set");
        assert_eq!(
            store
                .get_secret("db_key")
                .expect("get")
                .as_deref()
                .map(String::as_str),
            Some("super-secret")
        );
        store.delete_secret("db_key").expect("delete");
        assert!(store.get_secret("db_key").expect("get").is_none());
    }

    #[test]
    fn app_paths_resolve() {
        let paths = AppPaths::resolve().expect("OS dirs should resolve");
        assert!(paths.database_path().ends_with("keeftalk.db"));
    }
}

//! SQLCipher-encrypted local SQLite database, transactional migrations, and `.ktb` backups.

use std::path::Path;
use std::sync::{Arc, Mutex};

use keeftalk_crypto::Key256;
use keeftalk_platform::SecretStore;
use rusqlite::{Connection, OptionalExtension};
use thiserror::Error;
use tracing::info;
use zeroize::Zeroizing;

const MIGRATION_001: &str = include_str!("../../../migrations/001_initial.sql");
const DB_KEY_ACCOUNT: &str = "sqlcipher_db_key_v1";

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("database initialization failed: {0}")]
    Init(String),

    #[error("invalid database encryption key or corrupted database")]
    InvalidKey,

    #[error("database migration failed: {0}")]
    Migration(String),

    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("platform keyring error: {0}")]
    Platform(#[from] keeftalk_platform::PlatformError),

    #[error("crypto error: {0}")]
    Crypto(#[from] keeftalk_crypto::CryptoError),

    #[error("database lock poisoned")]
    LockPoisoned,
}

/// Manages retrieving or creating the 256-bit SQLCipher key in the OS credential store.
/// Never falls back to plaintext or hardcoded passphrases.
pub fn get_or_create_db_key(store: &dyn SecretStore) -> Result<Key256, DatabaseError> {
    if let Some(hex_key) = store.get_secret(DB_KEY_ACCOUNT)? {
        return decode_hex_key(&hex_key);
    }

    let key = Key256::random();
    let hex_key = Zeroizing::new(encode_hex_key(&key));
    store.set_secret(DB_KEY_ACCOUNT, &hex_key)?;
    Ok(key)
}

fn encode_hex_key(key: &Key256) -> String {
    let mut s = String::with_capacity(64);
    for b in key.as_bytes() {
        use std::fmt::Write as _;
        let _ = write!(s, "{b:02x}");
    }
    s
}

fn decode_hex_key(hex: &str) -> Result<Key256, DatabaseError> {
    let clean = hex.trim();
    if clean.len() != 64 {
        return Err(DatabaseError::InvalidKey);
    }
    let mut out = [0u8; 32];
    for (i, chunk) in clean.as_bytes().chunks_exact(2).enumerate() {
        let s = std::str::from_utf8(chunk).map_err(|_| DatabaseError::InvalidKey)?;
        out[i] = u8::from_str_radix(s, 16).map_err(|_| DatabaseError::InvalidKey)?;
    }
    Ok(Key256::from_bytes(out))
}

/// Thread-safe handle to an encrypted SQLCipher database.
#[derive(Clone)]
pub struct EncryptedDatabase {
    conn: Arc<Mutex<Connection>>,
}

impl EncryptedDatabase {
    /// Opens (or creates) an encrypted SQLCipher database at `path` and runs pending migrations.
    pub fn open(path: &Path, key: &Key256) -> Result<Self, DatabaseError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| DatabaseError::Init(format!("create db dir: {e}")))?;
        }
        let conn = Connection::open(path)?;
        Self::initialize_connection(conn, key)
    }

    /// Opens an in-memory SQLCipher database for isolated tests.
    pub fn open_in_memory(key: &Key256) -> Result<Self, DatabaseError> {
        let conn = Connection::open_in_memory()?;
        Self::initialize_connection(conn, key)
    }

    fn initialize_connection(conn: Connection, key: &Key256) -> Result<Self, DatabaseError> {
        let hex = Zeroizing::new(encode_hex_key(key));
        conn.pragma_update(None, "key", format!("x'{}'", &*hex))
            .map_err(|_| DatabaseError::InvalidKey)?;

        // Verify key immediately against sqlite_master
        let _: i64 = conn
            .query_row("SELECT count(*) FROM sqlite_master", [], |r| r.get(0))
            .map_err(|_| DatabaseError::InvalidKey)?;

        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;",
        )?;

        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&self) -> Result<(), DatabaseError> {
        let mut conn = self.conn.lock().map_err(|_| DatabaseError::LockPoisoned)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                applied_at_ms INTEGER NOT NULL
            );",
        )?;

        let migrations: &[(i64, &str, &str)] = &[(1, "001_initial", MIGRATION_001)];

        for &(version, name, sql) in migrations {
            let applied: Option<i64> = conn
                .query_row(
                    "SELECT version FROM schema_migrations WHERE version = ?1",
                    [version],
                    |r| r.get(0),
                )
                .optional()?;

            if applied.is_none() {
                info!(version, name, "Applying database migration");
                let tx = conn.transaction()?;
                tx.execute_batch(sql).map_err(|e| {
                    DatabaseError::Migration(format!("migration {version} ({name}): {e}"))
                })?;
                let now_ms = chrono::Utc::now().timestamp_millis();
                tx.execute(
                    "INSERT INTO schema_migrations (version, name, applied_at_ms) VALUES (?1, ?2, ?3)",
                    rusqlite::params![version, name, now_ms],
                )?;
                tx.commit()?;
            }
        }
        Ok(())
    }

    pub fn with_conn<F, T>(&self, f: F) -> Result<T, DatabaseError>
    where
        F: FnOnce(&Connection) -> Result<T, DatabaseError>,
    {
        let conn = self.conn.lock().map_err(|_| DatabaseError::LockPoisoned)?;
        f(&conn)
    }

    pub fn with_conn_mut<F, T>(&self, f: F) -> Result<T, DatabaseError>
    where
        F: FnOnce(&mut Connection) -> Result<T, DatabaseError>,
    {
        let mut conn = self.conn.lock().map_err(|_| DatabaseError::LockPoisoned)?;
        f(&mut conn)
    }

    /// Exports all user tables into an encrypted `.ktb` backup file without writing plaintext to disk.
    pub fn export_ktb_backup(&self, dest_path: &Path, password: &str) -> Result<(), DatabaseError> {
        let snapshot_json = self.with_conn(|conn| {
            let mut stmt = conn.prepare("SELECT id, title, markdown, tags_json, pinned, updated_at_ms FROM notes")?;
            let notes: Vec<NoteBackupRow> = stmt
                .query_map([], |r| {
                    Ok(NoteBackupRow {
                        id: r.get(0)?,
                        title: r.get(1)?,
                        markdown: r.get(2)?,
                        tags_json: r.get(3)?,
                        pinned: r.get::<_, i64>(4)? != 0,
                        updated_at_ms: r.get(5)?,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(serde_json_encode_notes(&notes))
        })?;

        let encrypted = keeftalk_crypto::backup::encrypt_ktb_bytes(snapshot_json.as_bytes(), password)?;
        std::fs::write(dest_path, encrypted)
            .map_err(|e| DatabaseError::Init(format!("write backup: {e}")))?;
        Ok(())
    }
}

struct NoteBackupRow {
    id: String,
    title: String,
    markdown: String,
    tags_json: String,
    pinned: bool,
    updated_at_ms: i64,
}

fn serde_json_encode_notes(rows: &[NoteBackupRow]) -> String {
    let mut out = String::from("{\"version\":1,\"notes\":[");
    for (idx, r) in rows.iter().enumerate() {
        if idx > 0 {
            out.push(',');
        }
        out.push_str(&format!(
            "{{\"id\":{:?},\"title\":{:?},\"markdown\":{:?},\"tags\":{},\"pinned\":{},\"updated_at_ms\":{}}}",
            r.id, r.title, r.markdown, r.tags_json, r.pinned, r.updated_at_ms
        ));
    }
    out.push_str("]}");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use keeftalk_platform::EphemeralSecretStore;

    #[test]
    fn sqlcipher_encrypts_file_and_rejects_wrong_key() {
        let dir = std::env::temp_dir().join(format!("keeftalk-db-test-{}", std::process::id()));
        let db_path = dir.join("test_cipher.db");
        let _ = std::fs::remove_dir_all(&dir);

        let key1 = Key256::random();
        let db = EncryptedDatabase::open(&db_path, &key1).expect("open encrypted db");
        db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO notes (id, owner_id, title, markdown, updated_at_ms) VALUES ('n1', 'u1', 'Secret Title', 'Classified Body', 1000)",
                [],
            )?;
            Ok(())
        })
        .expect("insert note");
        drop(db);

        // Verify plaintext does not appear in raw database file bytes
        let raw_bytes = std::fs::read(&db_path).expect("read db file");
        let raw_lossy = String::from_utf8_lossy(&raw_bytes);
        assert!(!raw_lossy.contains("Classified Body"));
        assert!(!raw_lossy.contains("Secret Title"));

        // Opening with wrong key must fail
        let wrong_key = Key256::random();
        assert!(matches!(
            EncryptedDatabase::open(&db_path, &wrong_key),
            Err(DatabaseError::InvalidKey)
        ));

        // Opening with correct key succeeds
        let reopened = EncryptedDatabase::open(&db_path, &key1).expect("reopen with right key");
        let title: String = reopened
            .with_conn(|conn| {
                conn.query_row("SELECT title FROM notes WHERE id='n1'", [], |r| r.get(0))
                    .map_err(DatabaseError::from)
            })
            .expect("query note");
        assert_eq!(title, "Secret Title");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn db_key_persists_in_secret_store() {
        let store = EphemeralSecretStore::default();
        let k1 = get_or_create_db_key(&store).expect("create key");
        let k2 = get_or_create_db_key(&store).expect("load key");
        assert_eq!(k1.as_bytes(), k2.as_bytes());
    }
}

//! Encrypted local SQLite/SQLCipher database interfaces for Keeftalk Desktop.
//!
//! Full SQLCipher storage, migrations, and backup/restore are implemented in Phase 3.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("database initialization failed: {0}")]
    Init(String),

    #[error("database migration failed: {0}")]
    Migration(String),

    #[error("query execution failed: {0}")]
    Query(String),
}

//! Cloudflare R2 presigned object storage interfaces for Keeftalk Desktop.
//!
//! Integrates with the `storage` Supabase Edge Function in Phase 5/6.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("cloud storage quota exceeded (prepaid balance empty after 1 GB free allowance)")]
    PrepaidBalanceEmpty,

    #[error("regional storage bucket unavailable: {0}")]
    InfrastructureUnavailable(String),

    #[error("storage transfer failed: {0}")]
    Transfer(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadTarget {
    pub url: String,
    pub object_key: String,
    pub bucket: String,
    pub country_code: String,
    pub region: String,
}

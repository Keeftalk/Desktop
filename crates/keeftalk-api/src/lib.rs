//! Supabase PostgREST, Edge Functions, and Realtime client boundary.
//!
//! Full HTTP/WebSocket transport is implemented in Phase 4.

use keeftalk_settings::BackendSection;
use thiserror::Error;
use url::Url;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("backend endpoint is not configured")]
    NotConfigured,

    #[error("invalid backend URL: {0}")]
    InvalidUrl(#[from] url::ParseError),

    #[error("request failed ({status}): {message}")]
    Http { status: u16, message: String },
}

#[derive(Debug, Clone)]
pub struct BackendEndpoints {
    pub base_url: Url,
    pub anon_key: String,
}

impl BackendEndpoints {
    pub fn from_settings(section: &BackendSection) -> Result<Self, ApiError> {
        if section.supabase_url.trim().is_empty() || section.supabase_anon_key.trim().is_empty() {
            return Err(ApiError::NotConfigured);
        }
        Ok(Self {
            base_url: Url::parse(&section.supabase_url)?,
            anon_key: section.supabase_anon_key.clone(),
        })
    }
}

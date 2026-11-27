//! Authentication, session lifecycle, and AEK recovery interfaces.
//!
//! Full Supabase GoTrue Auth + Argon2id AEK unwrap flow is implemented in Phase 4.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("authentication failed: {0}")]
    Failed(String),

    #[error("security recovery required to unlock Account Encryption Key")]
    RecoveryRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthState {
    LocalOffline,
    MockDemo { user_id: String, display_name: String },
    SignedOut,
    SignedIn { user_id: String, email: Option<String> },
    RecoveryRequired { user_id: String },
}

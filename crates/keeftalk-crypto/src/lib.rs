//! Cryptographic foundations and zeroized key wrappers for Keeftalk Desktop.
//!
//! Full Argon2id, HMAC-SHA256 domain separation, AES-256-GCM, PCK v4,
//! chunked R2 file encryption, and `.ktb` backup primitives are implemented in Phase 3.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("encryption failed: {0}")]
    Encryption(String),

    #[error("decryption failed or authentication tag mismatch")]
    Decryption,

    #[error("key derivation failed: {0}")]
    KeyDerivation(String),

    #[error("invalid ciphertext envelope: {0}")]
    InvalidFormat(String),
}

/// Fixed 256-bit symmetric key zeroized automatically on drop.
/// Deliberately omits `Debug` formatting of key material.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct Key256([u8; 32]);

impl Key256 {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl std::fmt::Debug for Key256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Key256([REDACTED])")
    }
}

/// Wire-compatible encrypted container matching Android `EncryptedObject`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncryptedObject {
    pub version: u32,
    pub key_id: String,
    pub iv: String,
    pub ciphertext: String,
    #[serde(default)]
    pub tag: Option<String>,
}

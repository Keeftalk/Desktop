//! Zero-knowledge cryptographic primitives and Keeftalk Android-compatible wire formats.
//!
//! Verified against `Keeftalk/Android` (`tn.keeftalk.chat.security.crypto`):
//! - Argon2id KDF (`m=65536 KiB`, `t=3`, `p=4`, `salt=16B`, `out=32B`)
//! - Domain separation via `HMAC-SHA256(AEK, info_utf8)`
//! - AES-256-GCM with 12-byte random IV and 16-byte appended auth tag
//! - Cloud-E2EE PCK v4 messages (`v4|{chatId}|{messageId}` AAD)
//! - Streaming chunked file encryption v2 (1 MiB chunks, `baseIv || chunkIndex`)
//! - Portable `.ktb` encrypted backups and Secrets colon-delimited format

pub mod aead;
pub mod backup;
pub mod files;
pub mod messaging;

use argon2::{Algorithm, Argon2, Params, Version};
use hmac::{Hmac, Mac};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

type HmacSha256 = Hmac<Sha256>;

/// Argon2id parameters matching `Argon2idManager.kt`.
pub const ARGON2_MEMORY_KIB: u32 = 65_536;
pub const ARGON2_ITERATIONS: u32 = 3;
pub const ARGON2_PARALLELISM: u32 = 4;
pub const SALT_LEN: usize = 16;
pub const KEY_LEN: usize = 32;

/// Domain separation labels matching `KeyManager.kt`.
pub const INFO_NOTES: &str = "Keeftalk/Notes/v1";
pub const INFO_AGENDA: &str = "Keeftalk/Agenda/v1";
pub const INFO_VAULT: &str = "Keeftalk/Vault/v1";
pub const INFO_CONV_WRAP: &str = "Keeftalk/ConversationWrap/v1";
pub const INFO_FILE_WRAP: &str = "Keeftalk/FileWrap/v1";

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("encryption failed: {0}")]
    Encryption(String),

    #[error("decryption failed or authentication tag mismatch")]
    Decryption,

    #[error("key derivation failed: {0}")]
    KeyDerivation(String),

    #[error("invalid ciphertext or envelope format: {0}")]
    InvalidFormat(String),

    #[error("AEK verification tag mismatch")]
    VerificationFailed,
}

/// Fixed 256-bit symmetric key zeroized automatically on drop.
#[derive(Clone, PartialEq, Eq, Zeroize, ZeroizeOnDrop)]
pub struct Key256([u8; KEY_LEN]);

impl Key256 {
    pub fn from_bytes(bytes: [u8; KEY_LEN]) -> Self {
        Self(bytes)
    }

    pub fn try_from_slice(slice: &[u8]) -> Result<Self, CryptoError> {
        let arr: [u8; KEY_LEN] = slice.try_into().map_err(|_| {
            CryptoError::InvalidFormat(format!("expected 32-byte key, got {} bytes", slice.len()))
        })?;
        Ok(Self(arr))
    }

    pub fn random() -> Self {
        let mut bytes = [0u8; KEY_LEN];
        rand::rngs::OsRng.fill_bytes(&mut bytes);
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; KEY_LEN] {
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

/// Generates a random 16-byte CSPRNG salt.
pub fn generate_salt() -> [u8; SALT_LEN] {
    let mut salt = [0u8; SALT_LEN];
    rand::rngs::OsRng.fill_bytes(&mut salt);
    salt
}

/// Derives a 256-bit Key Encryption Key (KEK) from UTF-8 password and salt using Argon2id.
pub fn derive_kek(password: &str, salt: &[u8]) -> Result<Key256, CryptoError> {
    if salt.len() < 8 {
        return Err(CryptoError::KeyDerivation(
            "salt must be at least 8 bytes".into(),
        ));
    }
    let params = Params::new(
        ARGON2_MEMORY_KIB,
        ARGON2_ITERATIONS,
        ARGON2_PARALLELISM,
        Some(KEY_LEN),
    )
    .map_err(|e| CryptoError::KeyDerivation(e.to_string()))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut out = [0u8; KEY_LEN];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut out)
        .map_err(|e| CryptoError::KeyDerivation(e.to_string()))?;
    Ok(Key256::from_bytes(out))
}

/// Derives a domain-separated subkey using `HMAC-SHA256(aek, info)` (matching `KeyManager.kt`).
pub fn derive_domain_key(aek: &Key256, info: &str) -> Result<Key256, CryptoError> {
    let mut mac = <HmacSha256 as Mac>::new_from_slice(aek.as_bytes())
        .map_err(|e| CryptoError::KeyDerivation(e.to_string()))?;
    mac.update(info.as_bytes());
    let bytes = mac.finalize().into_bytes();
    Key256::try_from_slice(&bytes)
}

/// Root Account Encryption Key (AEK) plus domain-separated subkeys.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct AccountKeyContext {
    aek: Key256,
}

impl std::fmt::Debug for AccountKeyContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AccountKeyContext([REDACTED])")
    }
}

impl AccountKeyContext {
    pub fn new(aek: Key256) -> Self {
        Self { aek }
    }

    pub fn generate() -> Self {
        Self {
            aek: Key256::random(),
        }
    }

    pub fn aek(&self) -> &Key256 {
        &self.aek
    }

    pub fn notes_key(&self) -> Result<Key256, CryptoError> {
        derive_domain_key(&self.aek, INFO_NOTES)
    }

    pub fn agenda_key(&self) -> Result<Key256, CryptoError> {
        derive_domain_key(&self.aek, INFO_AGENDA)
    }

    pub fn vault_key(&self) -> Result<Key256, CryptoError> {
        derive_domain_key(&self.aek, INFO_VAULT)
    }

    pub fn conversation_protection_key(&self) -> Result<Key256, CryptoError> {
        derive_domain_key(&self.aek, INFO_CONV_WRAP)
    }

    pub fn file_protection_key(&self) -> Result<Key256, CryptoError> {
        derive_domain_key(&self.aek, INFO_FILE_WRAP)
    }

    pub fn encrypt_aek_with_kek(&self, kek: &Key256) -> Result<EncryptedObject, CryptoError> {
        aead::wrap_key(&self.aek, kek)
    }

    pub fn decrypt_aek_with_kek(
        encrypted_aek: &EncryptedObject,
        kek: &Key256,
    ) -> Result<Self, CryptoError> {
        let aek = aead::unwrap_key(encrypted_aek, kek)?;
        Ok(Self { aek })
    }

    pub fn generate_verification_tag(&self) -> Result<EncryptedObject, CryptoError> {
        aead::encrypt(b"VERIFIED", &self.aek, None)
    }

    pub fn validate_verification_tag(&self, tag: &EncryptedObject) -> Result<(), CryptoError> {
        let plain = aead::decrypt(tag, &self.aek, None).map_err(|_| CryptoError::VerificationFailed)?;
        if plain.as_slice() == b"VERIFIED" {
            Ok(())
        } else {
            Err(CryptoError::VerificationFailed)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn argon2id_and_aek_wrap_round_trip() {
        let salt = [0x11u8; SALT_LEN];
        let kek = derive_kek("correct-horse-battery-staple", &salt).expect("kek");
        let kek2 = derive_kek("correct-horse-battery-staple", &salt).expect("kek deterministic");
        assert_eq!(kek.as_bytes(), kek2.as_bytes());

        let ctx = AccountKeyContext::generate();
        let wrapped = ctx.encrypt_aek_with_kek(&kek).expect("wrap aek");
        let recovered = AccountKeyContext::decrypt_aek_with_kek(&wrapped, &kek).expect("unwrap");
        assert_eq!(ctx.aek().as_bytes(), recovered.aek().as_bytes());

        let tag = recovered.generate_verification_tag().expect("tag");
        recovered.validate_verification_tag(&tag).expect("tag valid");

        let wrong_kek = derive_kek("wrong-password", &salt).expect("wrong kek");
        assert!(AccountKeyContext::decrypt_aek_with_kek(&wrapped, &wrong_kek).is_err());
    }

    #[test]
    fn domain_separation_keys_are_distinct_and_deterministic() {
        let aek = Key256::from_bytes([0x42u8; 32]);
        let ctx = AccountKeyContext::new(aek);
        let notes = ctx.notes_key().expect("notes");
        let agenda = ctx.agenda_key().expect("agenda");
        let vault = ctx.vault_key().expect("vault");
        let cpk = ctx.conversation_protection_key().expect("cpk");
        let fpk = ctx.file_protection_key().expect("fpk");

        assert_ne!(notes.as_bytes(), agenda.as_bytes());
        assert_ne!(notes.as_bytes(), vault.as_bytes());
        assert_ne!(cpk.as_bytes(), fpk.as_bytes());
        assert_eq!(notes.as_bytes(), ctx.notes_key().expect("notes2").as_bytes());
    }
}

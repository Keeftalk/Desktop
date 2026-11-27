//! Streaming 1 MiB chunked file encryption (v2) matching `StorageCryptoService.kt` & `FileUploadManager.kt`.

use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Nonce};
use hmac::{Hmac, Mac};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use crate::{CryptoError, EncryptedObject, Key256};

type HmacSha256 = Hmac<Sha256>;

pub const CHUNK_SIZE_BYTES: usize = 1024 * 1024; // 1 MiB
pub const BASE_IV_SIZE: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EnvelopeType {
    Owner,
    Conversation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncryptionEnvelope {
    pub recipient_id: String,
    pub wrapped_key: EncryptedObject,
    #[serde(rename = "type")]
    pub envelope_type: EnvelopeType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEncryptionMetadata {
    pub envelopes: Vec<EncryptionEnvelope>,
    pub file_iv: String,
    #[serde(default)]
    pub thumbnail_iv: Option<String>,
    #[serde(default)]
    pub encrypted_attributes: Option<EncryptedObject>,
    #[serde(default = "default_crypto_version")]
    pub crypto_version: u32,
    #[serde(default)]
    pub chunk_size: Option<usize>,
    #[serde(default)]
    pub plaintext_size: Option<u64>,
    #[serde(default)]
    pub thumbnail_plaintext_size: Option<u64>,
}

fn default_crypto_version() -> u32 {
    1
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileAttributes {
    pub file_name: Option<String>,
    pub mime_type: Option<String>,
    #[serde(default)]
    pub original_name: Option<String>,
}

pub fn generate_base_iv() -> [u8; BASE_IV_SIZE] {
    let mut iv = [0u8; BASE_IV_SIZE];
    rand::rngs::OsRng.fill_bytes(&mut iv);
    iv
}

fn chunk_nonce(base_iv: &[u8; BASE_IV_SIZE], chunk_index: i32) -> [u8; 12] {
    let mut nonce = [0u8; 12];
    nonce[..BASE_IV_SIZE].copy_from_slice(base_iv);
    nonce[BASE_IV_SIZE..].copy_from_slice(&chunk_index.to_be_bytes());
    nonce
}

fn chunk_aad(file_id: &str, chunk_index: i32) -> Vec<u8> {
    let mut aad = Vec::with_capacity(file_id.len() + 4);
    aad.extend_from_slice(file_id.as_bytes());
    aad.extend_from_slice(&chunk_index.to_be_bytes());
    aad
}

pub fn encrypt_chunk(
    plaintext_chunk: &[u8],
    fek: &Key256,
    base_iv: &[u8; BASE_IV_SIZE],
    chunk_index: i32,
    file_id: &str,
) -> Result<Vec<u8>, CryptoError> {
    let cipher = Aes256Gcm::new_from_slice(fek.as_bytes())
        .map_err(|e| CryptoError::Encryption(e.to_string()))?;
    let nonce = chunk_nonce(base_iv, chunk_index);
    let aad = chunk_aad(file_id, chunk_index);
    cipher
        .encrypt(
            Nonce::from_slice(&nonce),
            Payload {
                msg: plaintext_chunk,
                aad: &aad,
            },
        )
        .map_err(|e| CryptoError::Encryption(e.to_string()))
}

pub fn decrypt_chunk(
    encrypted_chunk: &[u8],
    fek: &Key256,
    base_iv: &[u8; BASE_IV_SIZE],
    chunk_index: i32,
    file_id: &str,
) -> Result<Vec<u8>, CryptoError> {
    let cipher = Aes256Gcm::new_from_slice(fek.as_bytes())
        .map_err(|_| CryptoError::Decryption)?;
    let nonce = chunk_nonce(base_iv, chunk_index);
    let aad = chunk_aad(file_id, chunk_index);
    cipher
        .decrypt(
            Nonce::from_slice(&nonce),
            Payload {
                msg: encrypted_chunk,
                aad: &aad,
            },
        )
        .map_err(|_| CryptoError::Decryption)
}

/// Computes per-user deduplication HMAC-SHA256(FPK, file_bytes).
pub fn hmac_file_hash(fpk: &Key256, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let mut mac = <HmacSha256 as Mac>::new_from_slice(fpk.as_bytes())
        .map_err(|e| CryptoError::KeyDerivation(e.to_string()))?;
    mac.update(data);
    Ok(mac.finalize().into_bytes().to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunked_file_encryption_round_trip_and_reorder_rejection() {
        let fek = Key256::random();
        let base_iv = generate_base_iv();
        let file_id = "file-uuid-001";

        let c0 = encrypt_chunk(b"chunk-zero", &fek, &base_iv, 0, file_id).expect("enc 0");
        let c1 = encrypt_chunk(b"chunk-one", &fek, &base_iv, 1, file_id).expect("enc 1");

        assert_eq!(decrypt_chunk(&c0, &fek, &base_iv, 0, file_id).expect("dec 0"), b"chunk-zero");
        assert_eq!(decrypt_chunk(&c1, &fek, &base_iv, 1, file_id).expect("dec 1"), b"chunk-one");

        // Swapped chunk index or wrong file_id must fail AAD/nonce verification
        assert!(decrypt_chunk(&c0, &fek, &base_iv, 1, file_id).is_err());
        assert!(decrypt_chunk(&c0, &fek, &base_iv, 0, "other-file").is_err());
    }
}

//! Standard AES-256-GCM authenticated encryption matching `StorageCryptoService.kt`.

use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use rand::RngCore;

use crate::{CryptoError, EncryptedObject, Key256};

pub const IV_SIZE: usize = 12;
pub const CRYPTO_VERSION: u32 = 1;

pub fn encrypt(
    plaintext: &[u8],
    key: &Key256,
    aad: Option<&[u8]>,
) -> Result<EncryptedObject, CryptoError> {
    let cipher = Aes256Gcm::new_from_slice(key.as_bytes())
        .map_err(|e| CryptoError::Encryption(e.to_string()))?;

    let mut iv = [0u8; IV_SIZE];
    rand::rngs::OsRng.fill_bytes(&mut iv);

    let payload = Payload {
        msg: plaintext,
        aad: aad.unwrap_or(&[]),
    };
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&iv), payload)
        .map_err(|e| CryptoError::Encryption(e.to_string()))?;

    Ok(EncryptedObject {
        version: CRYPTO_VERSION,
        key_id: "root".to_string(),
        iv: STANDARD.encode(iv),
        ciphertext: STANDARD.encode(ciphertext),
        tag: None,
    })
}

pub fn decrypt(
    encrypted: &EncryptedObject,
    key: &Key256,
    aad: Option<&[u8]>,
) -> Result<Vec<u8>, CryptoError> {
    let iv_bytes = STANDARD
        .decode(&encrypted.iv)
        .map_err(|e| CryptoError::InvalidFormat(format!("invalid base64 IV: {e}")))?;
    if iv_bytes.len() != IV_SIZE {
        return Err(CryptoError::InvalidFormat(format!(
            "expected {IV_SIZE}-byte IV, got {}",
            iv_bytes.len()
        )));
    }

    let mut ciphertext_bytes = STANDARD
        .decode(&encrypted.ciphertext)
        .map_err(|e| CryptoError::InvalidFormat(format!("invalid base64 ciphertext: {e}")))?;

    if let Some(tag_b64) = &encrypted.tag {
        let tag_bytes = STANDARD
            .decode(tag_b64)
            .map_err(|e| CryptoError::InvalidFormat(format!("invalid base64 tag: {e}")))?;
        ciphertext_bytes.extend_from_slice(&tag_bytes);
    }

    let cipher = Aes256Gcm::new_from_slice(key.as_bytes())
        .map_err(|_| CryptoError::Decryption)?;
    let payload = Payload {
        msg: &ciphertext_bytes,
        aad: aad.unwrap_or(&[]),
    };

    cipher
        .decrypt(Nonce::from_slice(&iv_bytes), payload)
        .map_err(|_| CryptoError::Decryption)
}

pub fn wrap_key(key_to_wrap: &Key256, wrapping_key: &Key256) -> Result<EncryptedObject, CryptoError> {
    encrypt(key_to_wrap.as_bytes(), wrapping_key, None)
}

pub fn unwrap_key(
    wrapped_key: &EncryptedObject,
    wrapping_key: &Key256,
) -> Result<Key256, CryptoError> {
    let raw = decrypt(wrapped_key, wrapping_key, None)?;
    Key256::try_from_slice(&raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aes_gcm_with_aad_round_trip_and_tamper_detection() {
        let key = Key256::random();
        let aad = b"v4|chat-1|msg-1";
        let enc = encrypt(b"hello keeftalk", &key, Some(aad)).expect("encrypt");
        let dec = decrypt(&enc, &key, Some(aad)).expect("decrypt");
        assert_eq!(dec, b"hello keeftalk");

        // Wrong AAD must fail
        assert!(decrypt(&enc, &key, Some(b"v4|chat-2|msg-1")).is_err());
        // Missing AAD must fail
        assert!(decrypt(&enc, &key, None).is_err());
    }
}

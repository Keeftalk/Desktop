//! Portable `.ktb` encrypted backup format (`BackupCryptoEngine.kt`)
//! and compact Secrets format (`SecretsRepositoryImpl.kt`).

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use rand::RngCore;

use crate::{CryptoError, EncryptedObject, Key256, SALT_LEN, aead, derive_kek, generate_salt};

const KTB_IV_LEN: usize = 12;
const KTB_MIN_LEN: usize = SALT_LEN + KTB_IV_LEN + 16;

/// Encrypts raw backup bytes into `.ktb` format: `[salt (16B)][iv (12B)][AES-256-GCM ciphertext+tag]`.
pub fn encrypt_ktb_bytes(plaintext: &[u8], password: &str) -> Result<Vec<u8>, CryptoError> {
    let salt = generate_salt();
    let key = derive_kek(password, &salt)?;

    let mut iv = [0u8; KTB_IV_LEN];
    rand::rngs::OsRng.fill_bytes(&mut iv);

    let cipher = Aes256Gcm::new_from_slice(key.as_bytes())
        .map_err(|e| CryptoError::Encryption(e.to_string()))?;
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&iv), plaintext)
        .map_err(|e| CryptoError::Encryption(e.to_string()))?;

    let mut out = Vec::with_capacity(SALT_LEN + KTB_IV_LEN + ciphertext.len());
    out.extend_from_slice(&salt);
    out.extend_from_slice(&iv);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

/// Decrypts `.ktb` bytes using the Argon2id backup password.
pub fn decrypt_ktb_bytes(encrypted: &[u8], password: &str) -> Result<Vec<u8>, CryptoError> {
    if encrypted.len() < KTB_MIN_LEN {
        return Err(CryptoError::InvalidFormat(
            "invalid .ktb backup length".into(),
        ));
    }
    let salt = &encrypted[..SALT_LEN];
    let iv = &encrypted[SALT_LEN..SALT_LEN + KTB_IV_LEN];
    let ciphertext = &encrypted[SALT_LEN + KTB_IV_LEN..];

    let key = derive_kek(password, salt)?;
    let cipher = Aes256Gcm::new_from_slice(key.as_bytes()).map_err(|_| CryptoError::Decryption)?;
    cipher
        .decrypt(Nonce::from_slice(iv), ciphertext)
        .map_err(|_| CryptoError::Decryption)
}

/// Encrypts a Secret credential/TOTP string into `{version}:{keyId}:{iv}:{ciphertext}` format.
pub fn encrypt_secret_compact(plaintext: &str, vault_key: &Key256) -> Result<String, CryptoError> {
    if plaintext.is_empty() {
        return Ok(String::new());
    }
    let obj = aead::encrypt(plaintext.as_bytes(), vault_key, None)?;
    Ok(format!(
        "{}:{}:{}:{}",
        obj.version, obj.key_id, obj.iv, obj.ciphertext
    ))
}

/// Decrypts a compact `{version}:{keyId}:{iv}:{ciphertext}` secret string.
pub fn decrypt_secret_compact(compact: &str, vault_key: &Key256) -> Result<String, CryptoError> {
    if compact.is_empty() {
        return Ok(String::new());
    }
    let parts: Vec<&str> = compact.split(':').collect();
    if parts.len() != 4 {
        return Err(CryptoError::InvalidFormat(
            "expected 4 colon-separated parts in secret payload".into(),
        ));
    }
    let version = parts[0]
        .parse::<u32>()
        .map_err(|_| CryptoError::InvalidFormat("invalid secret version".into()))?;
    let obj = EncryptedObject {
        version,
        key_id: parts[1].to_string(),
        iv: parts[2].to_string(),
        ciphertext: parts[3].to_string(),
        tag: None,
    };
    let bytes = aead::decrypt(&obj, vault_key, None)?;
    String::from_utf8(bytes)
        .map_err(|e| CryptoError::InvalidFormat(format!("secret is not valid UTF-8: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ktb_backup_round_trip() {
        let payload = br#"{"vault":"secret-data"}"#;
        let ktb = encrypt_ktb_bytes(payload, "backup-pass-123").expect("encrypt ktb");
        assert!(ktb.len() >= KTB_MIN_LEN);

        let dec = decrypt_ktb_bytes(&ktb, "backup-pass-123").expect("decrypt ktb");
        assert_eq!(dec, payload);
        assert!(decrypt_ktb_bytes(&ktb, "wrong-pass").is_err());
    }

    #[test]
    fn compact_secret_round_trip() {
        let key = Key256::random();
        let compact = encrypt_secret_compact("JBSWY3DPEHPK3PXP", &key).expect("enc");
        assert_eq!(compact.split(':').count(), 4);
        let dec = decrypt_secret_compact(&compact, &key).expect("dec");
        assert_eq!(dec, "JBSWY3DPEHPK3PXP");
    }
}

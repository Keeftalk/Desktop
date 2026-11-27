//! Cloud-E2EE Per-Conversation Key (PCK) and v4 message encryption.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::{AccountKeyContext, CryptoError, EncryptedObject, Key256, aead};

type HmacSha256 = Hmac<Sha256>;

pub const ENVELOPE_TYPE_PCK: u32 = 100;
pub const MESSAGE_CRYPTO_VERSION: u32 = 4;
const LEGACY_PCK_SALT: &[u8] = b"keeftalk-shared-pck-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncryptedMessageEnvelope {
    pub envelope_type: u32,
    pub crypto_version: u32,
    pub ciphertext_b64: String,
    pub nonce_b64: String,
}

/// Canonical AAD builder matching `MessageCryptoContext.kt`:
/// `v{version}|{chatId.lowercase().trim()}|{messageId.lowercase().trim()}`
pub fn build_message_aad(version: u32, chat_id: &str, message_id: &str) -> String {
    format!(
        "v{version}|{}|{}",
        chat_id.trim().to_ascii_lowercase(),
        message_id.trim().to_ascii_lowercase()
    )
}

pub fn encrypt_message_v4(
    pck: &Key256,
    chat_id: &str,
    message_id: &str,
    plaintext: &str,
) -> Result<EncryptedMessageEnvelope, CryptoError> {
    let aad = build_message_aad(MESSAGE_CRYPTO_VERSION, chat_id, message_id);
    let obj = aead::encrypt(plaintext.as_bytes(), pck, Some(aad.as_bytes()))?;
    Ok(EncryptedMessageEnvelope {
        envelope_type: ENVELOPE_TYPE_PCK,
        crypto_version: MESSAGE_CRYPTO_VERSION,
        ciphertext_b64: obj.ciphertext,
        nonce_b64: obj.iv,
    })
}

pub fn decrypt_message(
    pck: &Key256,
    chat_id: &str,
    message_id: &str,
    envelope: &EncryptedMessageEnvelope,
) -> Result<String, CryptoError> {
    if envelope.envelope_type != ENVELOPE_TYPE_PCK && envelope.envelope_type != 0 {
        return Err(CryptoError::InvalidFormat(format!(
            "unsupported envelope type {}",
            envelope.envelope_type
        )));
    }

    let aad_str = if envelope.crypto_version >= 4 {
        Some(build_message_aad(
            envelope.crypto_version,
            chat_id,
            message_id,
        ))
    } else {
        None
    };

    let obj = EncryptedObject {
        version: 1,
        key_id: "pck".to_string(),
        iv: envelope.nonce_b64.clone(),
        ciphertext: envelope.ciphertext_b64.clone(),
        tag: None,
    };

    let bytes = aead::decrypt(&obj, pck, aad_str.as_deref().map(str::as_bytes))?;
    String::from_utf8(bytes)
        .map_err(|e| CryptoError::InvalidFormat(format!("plaintext is not valid UTF-8: {e}")))
}

pub fn wrap_pck(
    pck: &Key256,
    ctx: &AccountKeyContext,
) -> Result<EncryptedObject, CryptoError> {
    let cpk = ctx.conversation_protection_key()?;
    aead::wrap_key(pck, &cpk)
}

pub fn unwrap_pck(
    encrypted_pck: &EncryptedObject,
    version: u32,
    ctx: &AccountKeyContext,
) -> Result<Key256, CryptoError> {
    if version >= 2 {
        let cpk = ctx.conversation_protection_key()?;
        aead::unwrap_key(encrypted_pck, &cpk)
    } else {
        aead::unwrap_key(encrypted_pck, ctx.aek())
    }
}

/// Legacy deterministic fallback PCK matching `ConversationKeyManager.createDeterministicKey`.
/// Retained solely for decrypting unexchanged historical conversations.
pub fn derive_legacy_deterministic_pck(conversation_id: &str) -> Result<Key256, CryptoError> {
    let normalized = conversation_id.trim().to_ascii_lowercase();
    let mut mac = <HmacSha256 as Mac>::new_from_slice(LEGACY_PCK_SALT)
        .map_err(|e| CryptoError::KeyDerivation(e.to_string()))?;
    mac.update(normalized.as_bytes());
    let bytes = mac.finalize().into_bytes();
    Key256::try_from_slice(&bytes)
}

pub fn decode_b64(input: &str) -> Result<Vec<u8>, CryptoError> {
    STANDARD
        .decode(input)
        .map_err(|e| CryptoError::InvalidFormat(format!("invalid base64: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_v4_aad_normalization_and_round_trip() {
        let pck = Key256::random();
        let env = encrypt_message_v4(
            &pck,
            "  CHAT-ABC-123 ",
            " MSG-XYZ-789 ",
            "Zero-knowledge message",
        )
        .expect("encrypt");

        // Decryption with lowercase trimmed IDs must match
        let plain = decrypt_message(&pck, "chat-abc-123", "msg-xyz-789", &env).expect("decrypt");
        assert_eq!(plain, "Zero-knowledge message");

        // Replay in a different chat must fail authentication
        assert!(decrypt_message(&pck, "other-chat", "msg-xyz-789", &env).is_err());
    }

    #[test]
    fn pck_wrap_and_legacy_fallback_work() {
        let ctx = AccountKeyContext::generate();
        let pck = Key256::random();
        let wrapped = wrap_pck(&pck, &ctx).expect("wrap");
        let unwrapped = unwrap_pck(&wrapped, 2, &ctx).expect("unwrap");
        assert_eq!(pck.as_bytes(), unwrapped.as_bytes());

        let legacy1 = derive_legacy_deterministic_pck("Chat-1").expect("legacy1");
        let legacy2 = derive_legacy_deterministic_pck(" chat-1 ").expect("legacy2");
        assert_eq!(legacy1.as_bytes(), legacy2.as_bytes());
    }
}

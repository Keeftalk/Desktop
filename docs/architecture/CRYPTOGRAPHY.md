# Keeftalk Cross-Platform Cryptographic Specification

This specification documents the exact cryptographic primitives, key hierarchy, encoding formats, and test-vector requirements verified against `Keeftalk/Android` (`tn.keeftalk.chat.security.crypto`).

**Rule:** Keeftalk Desktop (`keeftalk-crypto`) must remain bit-for-bit interoperable with Android ciphertexts for all synced user data.

---

## 1. Key Derivation Function (KDF)

Used in `Argon2idManager.kt` to derive the 256-bit **Key Encryption Key (KEK)** from the user's password:

| Parameter | Value |
|---|---|
| Algorithm | **Argon2id** (RFC 9106) |
| Memory (`m_cost`) | `65536` KiB (64 MiB) |
| Iterations (`t_cost`) | `3` |
| Parallelism (`p_cost`) | `4` |
| Salt Length | `16` bytes (CSPRNG) |
| Output Length | `32` bytes (256-bit AES key) |
| Password Encoding | UTF-8 bytes |

*Note:* Android includes a legacy fallback to `PBKDF2WithHmacSHA256` (10,000 iterations) only if native JNI Argon2 fails to load. Desktop uses pure-Rust `argon2` natively with zero fallback during key creation.

---

## 2. Key Hierarchy & Domain Separation

```text
User Password + 16B Salt
         │
     Argon2id
         ▼
        KEK (256-bit)
         │  AES-256-GCM unwraps `encrypted_account_key`
         ▼
        AEK (256-bit random root key)
         │
         ├── HMAC-SHA256(AEK, "Keeftalk/Notes/v1")            -> NotesKey
         ├── HMAC-SHA256(AEK, "Keeftalk/Agenda/v1")           -> AgendaKey (Calendar/Todo)
         ├── HMAC-SHA256(AEK, "Keeftalk/Vault/v1")            -> VaultKey (Vault/Secrets)
         ├── HMAC-SHA256(AEK, "Keeftalk/ConversationWrap/v1") -> CPK (wraps PCKs)
         └── HMAC-SHA256(AEK, "Keeftalk/FileWrap/v1")         -> FPK (wraps FEKs & dedup HMAC)
```

**Important Compatibility Detail (`KeyManager.kt` lines 326–332):**
Domain separation uses **raw HMAC-SHA256** with `AEK` as the HMAC key and the UTF-8 info string as the message (`Mac.getInstance("HmacSHA256")`), **not** HKDF-Expand.

### AEK Verification Tag
- Plaintext: UTF-8 bytes of `"VERIFIED"` (`[0x56, 0x45, 0x52, 0x49, 0x46, 0x49, 0x45, 0x44]`)
- Encrypted with `AEK` using standard AES-256-GCM (`EncryptedObject` JSON serialized into `user_security_settings.verification_tag`).

---

## 3. Standard Ciphertext Container (`EncryptedObject`)

All non-streaming AEAD payloads use **AES-256-GCM** with:
- Key size: `32` bytes (256 bits)
- Nonce / IV size: `12` bytes (96 bits, freshly generated via CSPRNG per encryption)
- Tag size: `16` bytes (128 bits), appended to ciphertext (`ciphertext || tag`)
- Encoding: Standard Base64 with padding (`=`), no line wraps (`Base64.NO_WRAP` / `RFC 4648` standard)

JSON representation:
```json
{
  "version": 1,
  "keyId": "root",
  "iv": "<base64-12-bytes>",
  "ciphertext": "<base64-ciphertext-plus-16-byte-tag>",
  "tag": null
}
```

---

## 4. Module-Specific Wire Formats

### 4.1 Messages (`public.messages`)
- **Envelope Type:** `100` (Cloud-E2EE PCK; type `0` treated as `100` for legacy compatibility)
- **Current Version:** `crypto_version = 4`
- **AAD (for `crypto_version >= 4`):**
  ```text
  v{version}|{chatId.to_ascii_lowercase().trim()}|{messageId.to_ascii_lowercase().trim()}
  ```
  Encoded as UTF-8 bytes. For `crypto_version <= 3`, AAD is empty (`None`).
- **Per-Conversation Key (`public.conversation_keys`):**
  - `version >= 2`: PCK unwrapped with `CPK` (`Keeftalk/ConversationWrap/v1`).
  - `version == 1`: PCK unwrapped directly with `AEK`.
  - **Legacy deterministic fallback** (only when no `conversation_keys` row exists):
    `HMAC-SHA256(key = b"keeftalk-shared-pck-v1", msg = conversation_id.to_ascii_lowercase().trim())`
- **Decrypted Payload:** Either raw UTF-8 message string or JSON `{"text": "...", "wrappedFek": "...", "mediaIv": "..."}`.

### 4.2 Streaming Chunked Files (v2, R2 Objects)
Used in `StorageCryptoService.kt` (`encryptChunk`/`decryptChunk`) and `FileUploadManager.kt`:
- **Chunk size:** `1,048,576` bytes (`1 MiB` plaintext per chunk; final chunk `<= 1 MiB`)
- **File Encryption Key (FEK):** Random `32`-byte AES key
- **Base IV:** Random `8`-byte prefix
- **Per-Chunk Nonce (`12` bytes):** `base_iv (8B) || chunk_index (4B big-endian i32)`
- **Per-Chunk AAD:** `file_id.as_bytes() (UTF-8) || chunk_index (4B big-endian i32)`
- **Per-Chunk Output:** `ciphertext || 16B GCM tag` (`plaintext_chunk_len + 16` bytes)
- **File Deduplication Hash:** `HMAC-SHA256(FPK, file_bytes)`
- **Metadata (`files.encryption_metadata`):** `FileEncryptionMetadata` containing `envelopes` (`OWNER` wrapped with `FPK`, `CONVERSATION` wrapped with `PCK`), `fileIv` (Base64 `baseIv`), optional `thumbnailIv`, and `encryptedAttributes` (JSON `FileAttributes` encrypted with `FEK`).

### 4.3 Notes (`public.notes`) & Calendar (`public.calendar_items`)
- Notes plaintext JSON: `{"title": "...", "content": "..."}` encrypted with `NotesKey` (`StoragePurpose::Notes`).
- Calendar plaintext JSON: `{"title": "...", "description": "..."}` encrypted with `AgendaKey` (`StoragePurpose::Agenda`).
- Stored remotely with placeholder `title = "[Encrypted]"` and `ciphertext`, `iv`, `crypto_version` columns.

### 4.4 Secrets (`secret_credentials`, `secret_totp_accounts`)
- Encrypted with `VaultKey` (`Keeftalk/Vault/v1`).
- Compact colon-delimited format (`SecretsRepositoryImpl.kt` line 50):
  ```text
  {version}:{keyId}:{base64_iv}:{base64_ciphertext}
  ```

### 4.5 Portable Encrypted Backups (`.ktb`)
Used in `BackupCryptoEngine.kt`:
- Binary layout:
  ```text
  [salt: 16 bytes][iv: 12 bytes][aes_256_gcm_ciphertext_and_tag: N + 16 bytes]
  ```
- Key derived via `Argon2id(password, salt)` with standard Keeftalk parameters.

---

## 5. Local Database Encryption

- Local SQLite database is encrypted at rest using **SQLCipher** (`PRAGMA key`) with a random 256-bit passphrase stored exclusively in the OS secure credential store (`keeftalk-platform`).
- Sensitive secrets (AEK, recovery tokens, OAuth tokens) use `zeroize` in Rust memory and are cleared on lock/logout.

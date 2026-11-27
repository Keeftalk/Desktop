# Keeftalk Desktop Threat Model

**Scope:** Desktop client (`Windows`, `Linux`, `macOS`), local encrypted storage, OS credential integration, and network communication with Keeftalk Supabase + Cloudflare R2.

---

## 1. Assets Protected
1. **User Encryption Keys:** Password-derived KEK, Account Encryption Key (AEK), domain keys, Per-Conversation Keys (PCKs), File Encryption Keys (FEKs), and local SQLCipher DB passphrase.
2. **User Content:** Chat messages, media attachments, Notes, Calendar events, Todo tasks, Vault files, Secrets (passwords & TOTP seeds), financial records.
3. **Authentication Material:** Supabase JWT access/refresh tokens, 2FA backup codes, IMAP/SMTP credentials.

---

## 2. Trust Boundaries & Adversaries

| Adversary | Assumed Capability | Mitigation |
|---|---|---|
| **Compromised Cloud / Malicious Backend Admin** | Full read/write access to Supabase PostgreSQL & Cloudflare R2 buckets. | Zero-knowledge client-side AES-256-GCM encryption before upload; AEK wrapped client-side with Argon2id KEK; RLS enforces per-user isolation on metadata. |
| **Passive Network Attacker** | Observes/tampers with network traffic. | TLS 1.2+/HTTPS + WSS everywhere; AEAD integrity tags detect ciphertext/AAD tampering (`v4\|chatId\|messageId`). |
| **Offline Local Disk Thief** | Reads user home/AppData directory while app is locked or device powered off. | Local SQLite encrypted with SQLCipher; DB passphrase and wrapped AEK stored in OS Keychain / Credential Manager / Secret Service (never plaintext). |
| **Shoulder Surfer / Shared Desktop** | Physical access while user steps away. | Configurable auto-lock timeout, vault password lock, and automatic clipboard clearing after timeout for copied secrets. |

---

## 3. Known Ecosystem Risks & Mitigations

1. **Legacy Deterministic PCK Fallback (`ConversationKeyManager`):**
   - *Finding:* Android derives a deterministic fallback key from `HMAC-SHA256("keeftalk-shared-pck-v1", conversationId)` when no random PCK exists in `conversation_keys`. Anyone knowing `conversationId` (including a server admin) could derive that fallback key.
   - *Desktop Policy:* Support decrypting historical messages encrypted under the legacy fallback for Android interoperability, clearly flag unexchanged fallback chats in Security Center diagnostics, and generate cryptographically random CPK-wrapped PCKs wherever permitted.
2. **Metadata Exposure:**
   - Relational metadata (`chat_id`, `sender_id`, `timestamp`, `owner_id`, `file_size`, `storage_path`) remains visible to Supabase for routing, RLS, and pay-as-you-go billing metering (`docs/security/METADATA_EXPOSURE.md`).
3. **Clipboard Leakage:**
   - Copied passwords/TOTP codes must clear automatically after a configurable timeout (default 30s) if the clipboard still holds the copied secret.
4. **Memory Exposure:**
   - All key structs in `keeftalk-crypto` implement `Zeroize` + `ZeroizeOnDrop`. Sensitive values are never formatted in `Debug` impls or `tracing` logs.
5. **No Plaintext Credential Fallback:**
   - Unlike Android's emergency DB fallback string, Desktop **refuses** to silently fall back to hardcoded or plaintext passphrases if the OS keychain fails; it surfaces an explicit recovery/unlock prompt.

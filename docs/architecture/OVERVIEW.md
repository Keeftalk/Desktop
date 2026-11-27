# Keeftalk Desktop — System Architecture Overview

Keeftalk Desktop is a Rust-native, offline-first, zero-knowledge desktop Super App compatible with the Keeftalk Android and Supabase/R2 backend ecosystem.

---

## 1. Design Principles

1. **Zero-Knowledge Privacy:** Plaintext user content and encryption keys never leave the device unencrypted.
2. **Offline-First Sovereignty:** All local features operate 100% offline against an encrypted local SQLite/SQLCipher database without requiring a cloud account.
3. **Strict Layer Isolation:** The Iced UI layer communicates exclusively via typed commands/events with `keeftalk-core` services and never directly accesses SQLite, cryptographic keys, or HTTP endpoints.
4. **Backend Compatibility:** Integrates with the existing `Keeftalk/Backend` Supabase Auth, PostgREST, Realtime, Edge Functions (`storage`, `billing-*`, `two-factor-auth`, `link-preview`), and Cloudflare R2 contracts.
5. **Self-Hostable & Open-Source First:** Zero hardcoded production lock-in; includes an isolated local mock/demo mode for contributors without backend credentials.

---

## 2. Rust Workspace Crate Topology

```text
Desktop/crates/
├── keeftalk-app            # Binary entry point, CLI/env bootstrap, tracing, DI wiring
├── keeftalk-ui             # Iced GUI shell, themes, navigation, command palette, views
├── keeftalk-core           # Application services, domain models, command/event bus, mock mode
├── keeftalk-crypto         # Argon2id, AEK/KEK/domain keys, AES-256-GCM, PCK, chunked files, .ktb
├── keeftalk-database       # SQLCipher-encrypted SQLite pool, migrations, DAOs, backup/restore
├── keeftalk-auth           # Supabase GoTrue client, session lifecycle, AEK recovery/setup, 2FA
├── keeftalk-sync           # Durable offline outbox queues, retry/backoff, conflict resolution
├── keeftalk-api            # Supabase PostgREST, Realtime WebSocket, Edge Functions HTTP client
├── keeftalk-storage        # R2 presigned client (via storage Edge Function) & chunked streaming
├── keeftalk-notifications  # OS native desktop notifications & audio alerts
├── keeftalk-settings       # Layered TOML/env config, theme/module preferences
└── keeftalk-platform       # OS abstractions: keyring/Keychain/DPAPI, XDG/AppData paths, clipboard
```

### Dependency Direction (Acyclic DAG)

```text
                 keeftalk-app
                /      |      \
      keeftalk-ui   keeftalk-core   keeftalk-platform
                    /   |    |    \
         keeftalk-auth  |  keeftalk-sync   keeftalk-notifications
              |         |     |     \
        keeftalk-api    |  keeftalk-storage
              \         |     /
          keeftalk-database  keeftalk-settings
                    \       /
                  keeftalk-crypto
```

- `keeftalk-ui` depends ONLY on `keeftalk-core` (domain DTOs/messages) and `keeftalk-settings` (theme/UI config). It has **zero** dependency on `keeftalk-database`, `keeftalk-crypto`, `keeftalk-api`, or `sqlx`/`rusqlite`.

---

## 3. Data & Sync Architecture

```text
   Iced UI (View + Update)
           │  Typed Commands / State Snapshots
           ▼
   Application Core (keeftalk-core)
           │
     ┌─────┴──────────────────┐
     ▼                        ▼
Local Repository        Sync Engine (keeftalk-sync)
     │                        │
     ▼                        ▼
SQLCipher DB            Durable Outbox Queue
(keeftalk-database)           │
                        ┌─────┴───────────┐
                        ▼                 ▼
                  Supabase API      R2 Presigned Storage
                  (keeftalk-api)    (keeftalk-storage)
```

### Synchronization States
Every syncable entity tracks an explicit state:
- `LocalOnly` (offline/unauthenticated mode)
- `PendingUpload` (queued in local durable outbox)
- `Synchronizing` (in-flight network request)
- `Synchronized` (confirmed persisted remotely)
- `ConflictDetected` (remote `updated_at` / version diverged; surfaced safely without silent overwrite)
- `SyncFailed` (transient or auth/quota error with exponential backoff retry)

---

## 4. Platform Abstraction (`keeftalk-platform`)

| Capability | Windows 10/11 | Linux | macOS |
|---|---|---|---|
| **Master / DB Key Storage** | Windows Credential Manager (`keyring` / Win32 CredWrite) | Secret Service D-Bus (`libsecret` / KWallet) | macOS Keychain |
| **App Data / DB Path** | `%APPDATA%\Keeftalk` | `$XDG_DATA_HOME/keeftalk` | `~/Library/Application Support/Keeftalk` |
| **Config Path** | `%APPDATA%\Keeftalk\config` | `$XDG_CONFIG_HOME/keeftalk` | `~/Library/Application Support/Keeftalk` |
| **Notifications** | WinRT Toast (`notify-rust`) | freedesktop D-Bus (`notify-rust`) | macOS UserNotifications |
| **Clipboard Auto-Clear** | Timed background zero/clear if unchanged | Timed clear if unchanged | Timed clear if unchanged |

**Security Rule:** If the OS secure credential provider is unavailable, `keeftalk-platform` returns an explicit error and refuses to silently fall back to plaintext disk storage.

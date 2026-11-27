# Keeftalk Desktop — Feature Compatibility Matrix

> **Rule:** Features are marked `Implemented` only after their real local/backend logic and automated tests pass—never merely for UI stubs.

## Release 1 — Desktop MVP Scope

| Module | Group | Windows | Linux | macOS | Offline | Cloud Sync | Status |
|---|---|---|---|---|---|---|---|
| Application Shell & Themes (`keeftalk-ui`) | Core | CI Configured | Verified | CI Configured | Yes | N/A | **Implemented** |
| Local Encrypted DB (SQLCipher, `keeftalk-database`) | Core | CI Configured | Verified | CI Configured | Yes | N/A | **Implemented** |
| Cryptographic Primitives (`keeftalk-crypto`) | Security | CI Configured | Verified | CI Configured | Yes | N/A | **Implemented** |
| Authentication & AEK Recovery (`keeftalk-auth`, `keeftalk-api`) | Security | CI Configured | Verified | CI Configured | Local Mode | Supabase Auth | **Implemented** |
| Offline Sync Outbox (`keeftalk-sync`) | Core | CI Configured | Verified | CI Configured | Yes | Optional | **Implemented** |
| **My Chats** (E2EE PCK v4, SQLCipher persistence, outbox) | Communication | CI Configured | Verified | CI Configured | Yes | Outbox Wired | **Implemented (Core)** |
| **My Notes** (Markdown preview, NotesKey E2EE, SQLCipher) | Productivity | CI Configured | Verified | CI Configured | Yes | Outbox Wired | **Implemented (Core)** |
| **My Todo** (Tasks, priorities, completion, SQLCipher) | Productivity | CI Configured | Verified | CI Configured | Yes | Outbox Wired | **Implemented (Core)** |
| **My Calendar** (Events, AgendaKey E2EE, SQLCipher) | Productivity | CI Configured | Verified | CI Configured | Yes | Outbox Wired | **Implemented (Core)** |
| **My Vault** (Encrypted entries, `.ktb` backup export) | Security | CI Configured | Verified | CI Configured | Yes | R2 Client Wired | **Implemented (Core)** |
| Desktop Notifications (`keeftalk-notifications`) | Core | CI Configured | Verified | CI Configured | Yes | Realtime | Foundation |

## Release 2+ — Extended Super App Modules

| Module | Group | Windows | Linux | macOS | Offline | Cloud Sync | Notes |
|---|---|---|---|---|---|---|---|
| **My Secrets** | Security | Planned | Planned | Planned | Yes | Optional | Phase 7 |
| **Security Center** | Security | Planned | Planned | Planned | Yes | Optional | Phase 7 |
| **My Habits** | Productivity | Planned | Planned | Planned | Yes | Optional | Phase 7 |
| **My Docs** | Productivity | Planned | Planned | Planned | Yes | Optional | Phase 7 |
| **My Books** | Productivity | Planned | Planned | Planned | Yes | Optional | Phase 7 |
| **My Calls** | Communication | Planned | Planned | Planned | No | Required | WebRTC (`call_sessions`/`call_signaling`) |
| **My Finances & Wallet** | Finance | Planned | Planned | Planned | Yes | Optional | Phase 7 |
| **My E-Mail** | Communication | Planned | Planned | Planned | Cached | IMAP/SMTP | Phase 7 |
| **My Feed** | Discover | Planned | Planned | Planned | Yes | Optional | Phase 7 |
| **My Market** | Finance | Planned | Planned | Planned | Cached | Market APIs | Phase 7 |
| **My Maps** | Personal | Planned | Planned | Planned | Cached | Optional | Phase 7 |
| **My Din** | Personal | Planned | Planned | Planned | Yes | N/A | Phase 7 |
| **My Browser** | Discover | Limited | Limited | Limited | No | Optional | Privacy-focused external integration |
| **My SMS/MMS** | Communication | Unsupported | Unsupported | Unsupported | Read-only | Optional | Desktop OSes lack carrier SIM telephony APIs |

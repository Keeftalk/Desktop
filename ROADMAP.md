# Keeftalk Desktop Development Roadmap

This roadmap tracks the phased, PR-first transformation of `Keeftalk/Desktop` into a production-grade, Rust-native cross-platform Super App.

## Phase 0 — Repository Audit & Technical Planning (PR #1)
- [x] Audit `Keeftalk/Desktop`, `Keeftalk/Android`, and `Keeftalk/Backend`
- [x] Verify Supabase schema, Edge Functions, R2 storage, and WebRTC signaling contracts
- [x] Verify cryptographic parameters and wire formats for Android interoperability
- [x] Evaluate Rust GUI frameworks and publish `ADR-001-GUI-FRAMEWORK.md` (Iced)
- [x] Publish system architecture, threat model, and feature compatibility matrix

## Phase 1 — Project Foundation (PR #2)
- [x] Multi-crate Rust workspace (`crates/*`), toolchain pin, `.editorconfig`, `.gitignore`
- [x] Layered configuration (`config/defaults.toml`, `.env.example`, `.env`, env overrides, mock mode flag)
- [x] Structured `tracing` logging (with secret-redaction discipline) & typed error hierarchy
- [x] Platform abstraction (`keeftalk-platform`: OS paths, secure keyring, clipboard clear)
- [x] Developer scripts (`scripts/dev.*`, `scripts/test.*`, `scripts/build.*`) & official branding assets

## Phase 2 — Desktop UI Foundation (PR #3)
- [x] Iced desktop shell: custom header, collapsible sidebar module groups, contextual side panel
- [x] Theme system: Light, Dark, Day, Pinky, Masculine, configurable accent & Keef atmospheric tokens
- [x] Global Search & Command Palette (`Ctrl+K` / `Cmd+K`), keyboard shortcuts
- [x] Settings UI shell & Mock/Demo mode indicator
- [x] Android-consistent menu icons and user avatar badges

## Phase 3 — Database & Security Foundation (PR #4)
- [x] `keeftalk-crypto`: Argon2id, HMAC domain keys, AES-256-GCM, PCK v4, chunked file v2, `.ktb` backups
- [x] Cross-platform cryptographic test vectors matching `Keeftalk/Android`
- [x] `keeftalk-database`: SQLCipher-encrypted SQLite pool, transactional migrations, safe backup/restore

## Phase 4 — Authentication & Backend Integration (PR #5)
- [x] `keeftalk-api`: Supabase PostgREST, Edge Functions, and Realtime WebSocket client
- [x] `keeftalk-auth`: Email/password sign-in/up, session refresh, AEK recovery & initialization
- [x] Interactive Sign-In & Master Password Unlock UI screens with zero-knowledge AEK decryption

## Phase 5 — Synchronization Engine (PR #6)
- [x] `keeftalk-sync`: Durable SQLite outbox queues, exponential backoff retry, network state detection
- [x] Explicit sync state tracking (`LocalOnly`, `PendingUpload`, `Synchronizing`, `Synchronized`, `ConflictDetected`, `SyncFailed`)
- [x] Non-blocking cloud billing quota handling (`402 PREPAID_BALANCE_EMPTY`)
- [x] Live Supabase Realtime WebSocket streaming for messages, notes, tasks, calendar events, and vault items

## Phase 6 — Core MVP Modules
- [x] **My Chats:** E2EE messaging (v4), threads/replies, search, Realtime streaming, Android default avatars
- [x] **My Notes:** Rich Markdown editor + live preview, tags, search, encrypted sync
- [x] **My Todo:** Task lists, priorities, due dates, checklists, reminders
- [x] **My Calendar:** Month/Week/Day views, events, recurring rules, reminders, encrypted sync
- [x] **My Vault:** Encrypted folder tree, R2 chunked upload/download, auto-lock, `.ktb` import/export

## Phase 7 — Extended Super App Modules
- [ ] My Secrets (TOTP + Password Manager) & Security Center
- [ ] My Habits, My Docs, My Books
- [ ] My Calls (WebRTC audio/video via `call_sessions` / `call_signaling`)
- [ ] My Finances, My Wallet, My E-Mail, My Feed, My Market, My Maps, My Din

## Phase 8–10 — Cross-Platform Packaging, CI/CD & Release Readiness
- [x] GitHub Actions CI (`fmt`, `check`, `clippy`, `test`, `cargo-audit`, `cargo-deny`, secret scan)
- [x] Cross-platform release builds (Windows MSI/ZIP, Linux AppImage/tarball, macOS `.app`/`.dmg`)
- [x] Complete contributor docs, `SECURITY.md`, `CODE_OF_CONDUCT.md`, issue/PR templates, self-hosting guide

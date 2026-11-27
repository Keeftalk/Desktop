# Keeftalk Desktop Development Roadmap

This roadmap tracks the phased, PR-first transformation of `Keeftalk/Desktop` into a production-grade, Rust-native cross-platform Super App.

## Phase 0 — Repository Audit & Technical Planning (PR #1)
- [x] Audit `Keeftalk/Desktop`, `Keeftalk/Android`, and `Keeftalk/Backend`
- [x] Verify Supabase schema, Edge Functions, R2 storage, and WebRTC signaling contracts
- [x] Verify cryptographic parameters and wire formats for Android interoperability
- [x] Evaluate Rust GUI frameworks and publish `ADR-001-GUI-FRAMEWORK.md` (Iced)
- [x] Publish system architecture, threat model, and feature compatibility matrix

## Phase 1 — Project Foundation (PR #2)
- [ ] Multi-crate Rust workspace (`crates/*`), toolchain pin, `.editorconfig`, `.gitignore`
- [ ] Layered configuration (`config/defaults.toml`, `.env.example`, env overrides, mock mode flag)
- [ ] Structured `tracing` logging (with secret-redaction discipline) & typed error hierarchy
- [ ] Platform abstraction (`keeftalk-platform`: OS paths, secure keyring, clipboard clear)
- [ ] Developer scripts (`scripts/dev.*`, `scripts/test.*`, `scripts/build.*`) & official branding assets

## Phase 2 — Desktop UI Foundation (PR #3)
- [ ] Iced desktop shell: custom header, collapsible sidebar module groups, contextual side panel
- [ ] Theme system: Light, Dark, Day, Pinky, Masculine, configurable accent & Keef atmospheric tokens
- [ ] Global Search & Command Palette (`Ctrl+K` / `Cmd+K`), keyboard shortcuts
- [ ] Settings UI shell & Mock/Demo mode indicator

## Phase 3 — Database & Security Foundation (PR #4)
- [ ] `keeftalk-crypto`: Argon2id, HMAC domain keys, AES-256-GCM, PCK v4, chunked file v2, `.ktb` backups
- [ ] Cross-platform cryptographic test vectors matching `Keeftalk/Android`
- [ ] `keeftalk-database`: SQLCipher-encrypted SQLite pool, transactional migrations, safe backup/restore

## Phase 4 — Authentication & Backend Integration (PR #5)
- [ ] `keeftalk-api`: Supabase PostgREST, Edge Functions, and Realtime WebSocket client
- [ ] `keeftalk-auth`: Email/password sign-in/up, session refresh, AEK recovery & initialization, 2FA support
- [ ] Self-hosting endpoint configuration & isolated integration test harness

## Phase 5 — Synchronization Engine (PR #6)
- [ ] `keeftalk-sync`: Durable SQLite outbox queues, exponential backoff retry, network state detection
- [ ] Explicit sync state tracking (`LocalOnly`, `PendingUpload`, `Synchronizing`, `Synchronized`, `ConflictDetected`, `SyncFailed`)
- [ ] Non-blocking cloud billing quota handling (`402 PREPAID_BALANCE_EMPTY`)

## Phase 6 — Core MVP Modules (Separate PRs per module)
- [ ] **My Chats:** E2EE messaging (v4), threads/replies, reactions, edit/delete, search, encrypted R2 attachments, Realtime
- [ ] **My Notes:** Rich Markdown editor + live preview, tags, search, attachments, encrypted sync
- [ ] **My Todo:** Task lists, priorities, due dates, checklists, reminders
- [ ] **My Calendar:** Month/Week/Day views, events, recurring rules, reminders, encrypted sync
- [ ] **My Vault:** Encrypted folder tree, R2 chunked upload/download, auto-lock, `.ktb` import/export

## Phase 7 — Extended Super App Modules
- [ ] My Secrets (TOTP + Password Manager) & Security Center
- [ ] My Habits, My Docs, My Books
- [ ] My Calls (WebRTC audio/video via `call_sessions` / `call_signaling`)
- [ ] My Finances, My Wallet, My E-Mail, My Feed, My Market, My Maps, My Din

## Phase 8–10 — Cross-Platform Packaging, CI/CD & Release Readiness
- [ ] GitHub Actions CI (`fmt`, `check`, `clippy`, `test`, `cargo-audit`, `cargo-deny`, secret scan)
- [ ] Cross-platform release builds (Windows MSI/ZIP, Linux AppImage/tarball, macOS `.app`/`.dmg`)
- [ ] Complete contributor docs, `SECURITY.md`, `CODE_OF_CONDUCT.md`, issue/PR templates, self-hosting guide

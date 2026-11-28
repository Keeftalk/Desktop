# Changelog

All notable changes to **Keeftalk Desktop** are documented in this file.

## [Unreleased]

### Added
- Phase 0: Ecosystem audit (`Desktop`, `Android`, `Backend`), architecture overview, `ADR-001-GUI-FRAMEWORK.md` (Iced), cross-platform cryptography spec, threat model, feature compatibility matrix, and roadmap.
- Phase 1: 12-crate Rust workspace, layered TOML/env configuration (`keeftalk-settings`), cross-platform OS abstractions (`keeftalk-platform`), core domain models & local mock/demo mode (`keeftalk-core`), official branding/audio/emoji assets, and developer setup scripts.
- Phase 2: Iced desktop shell (`keeftalk-ui`), 5 built-in theme variants (Light, Dark, Day, Pinky, Masculine), customizable accent colors, global Command Palette (`Ctrl+K`), Android-consistent menu icons, and user avatar badges.
- Phase 3: Zero-knowledge cryptographic suite (`keeftalk-crypto`: Argon2id, HMAC-SHA256 subkeys, AES-256-GCM, PCK v4, chunked file encryption, `.ktb` backups) and SQLCipher encrypted database (`keeftalk-database`).
- Phase 4: Supabase REST/Edge Functions client (`keeftalk-api`), GoTrue authentication and Account Encryption Key (AEK) cloud recovery (`keeftalk-auth`), plus interactive Sign-In and Master Password Unlock UI screens.
- Phase 5: Durable SQLCipher outbox sync engine (`keeftalk-sync`) with exponential backoff and Supabase Realtime WebSocket streaming for live Postgres table changes.
- Phase 6: Core MVP Super App modules (My Chats, My Notes, My Todo, My Calendar, My Vault) with offline-first persistence and E2EE cloud sync.

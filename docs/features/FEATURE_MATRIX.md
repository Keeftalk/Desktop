# Keeftalk Desktop — Feature Compatibility Matrix

> **Rule:** Features are only marked `Implemented` after their real backend/local logic, persistence, and tests exist and pass—never merely because a UI stub exists.

## Release 1 — Desktop MVP Scope

| Module | Group | Windows | Linux | macOS | Offline | Cloud Sync | Status |
|---|---|---|---|---|---|---|---|
| Application Shell & Themes | Core | Planned | Planned | Planned | Yes | N/A | Phase 1–2 |
| Local Encrypted DB (SQLCipher) | Core | Planned | Planned | Planned | Yes | N/A | Phase 3 |
| Authentication & AEK Recovery | Security | Planned | Planned | Planned | Local Mode | Supabase Auth | Phase 4 |
| Offline Sync Engine & Queues | Core | Planned | Planned | Planned | Yes | Optional | Phase 5 |
| **My Chats** (E2EE text, threads, reactions, attachments, search) | Communication | Planned | Planned | Planned | Yes | Optional | Phase 6 |
| **My Notes** (Markdown + preview, tags, search, attachments) | Productivity | Planned | Planned | Planned | Yes | Optional | Phase 6 |
| **My Todo** (Lists, priorities, due dates, checklists, reminders) | Productivity | Planned | Planned | Planned | Yes | Optional | Phase 6 |
| **My Calendar** (Month/Week/Day views, events, reminders, recurring) | Productivity | Planned | Planned | Planned | Yes | Optional | Phase 6 |
| **My Vault** (Encrypted files, folders, `.ktb` import/export, lock) | Security | Planned | Planned | Planned | Yes | Optional (R2) | Phase 6 |
| Desktop Notifications & Audio Alerts | Core | Planned | Planned | Planned | Yes | Realtime | Phase 8 |

## Release 2+ — Extended Super App Modules

| Module | Group | Windows | Linux | macOS | Offline | Cloud Sync | Notes |
|---|---|---|---|---|---|---|---|
| **My Secrets** (Passwords + TOTP + Clipboard auto-clear) | Security | Planned | Planned | Planned | Yes | Optional | Phase 7 |
| **Security Center** (Sessions, DB backup/restore, key status) | Security | Planned | Planned | Planned | Yes | Optional | Phase 7 |
| **My Habits** (Schedules, streaks, heatmaps, stats) | Productivity | Planned | Planned | Planned | Yes | Optional | Phase 7 |
| **My Docs** (PDF, Markdown, Mermaid, images, search) | Productivity | Planned | Planned | Planned | Yes | Optional | Phase 7 |
| **My Books** (EPUB/PDF reader, bookmarks, Reading XP) | Productivity | Planned | Planned | Planned | Yes | Optional | Phase 7 |
| **My Calls** (WebRTC Audio/Video, device selection, call history) | Communication | Planned | Planned | Planned | No | Required | Phase 7 |
| **My Finances & Wallet** (Income/expense, budgets, assets) | Finance | Planned | Planned | Planned | Yes | Optional | Phase 7 |
| **My E-Mail** (IMAP/SMTP client, threads, attachments) | Communication | Planned | Planned | Planned | Cached | IMAP/SMTP | Phase 7 |
| **My Feed** (RSS/Atom reader, article extraction) | Discover | Planned | Planned | Planned | Yes | Optional | Phase 7 |
| **My Market** (Stocks, crypto tickers, watchlists, charts) | Finance | Planned | Planned | Planned | Cached | Market APIs | Phase 7 |
| **My Maps** (Saved places, map view) | Personal | Planned | Planned | Planned | Cached | Optional | Phase 7 |
| **My Din** (Prayer times, Qibla, Hijri calendar, Quran/Douaa) | Personal | Planned | Planned | Planned | Yes | N/A | Phase 7 |
| **My Browser** | Discover | Limited | Limited | Limited | No | Optional | Evaluates safe external/privacy integration; no unmaintained embedded engine |
| **My SMS/MMS** | Communication | Unsupported | Unsupported | Unsupported | Read-only (synced) | Optional | Desktop OSes lack carrier SIM telephony APIs |

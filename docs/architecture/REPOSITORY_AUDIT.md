# Keeftalk Ecosystem Repository Audit

**Date:** 2026-11-27  
**Auditor:** Keeftalk Desktop Engineering  
**Authoritative References:**
- `Keeftalk/Desktop` (`d4216ce`)
- `Keeftalk/Android` (`4a5b273`)
- `Keeftalk/Backend` (`aeb212e`)

---

## 1. Existing Desktop Repository (`Keeftalk/Desktop`)

| Item | Finding |
|---|---|
| Git history | Single initial commit `d4216ce` on `main` |
| Files | `README.md` (`# Desktop`) only |
| Existing code / CI | None |
| Reusable desktop code | None (clean-slate Rust workspace) |

---

## 2. Android Reference Repository (`Keeftalk/Android`)

### 2.1 Architecture & Stack
- **Language / UI:** Kotlin, Jetpack Compose, Material 3, Navigation3, custom Keef Glassmorphic Design System (`KeefDesignTokens`, `KeefHomeView`, `KeefThemeBackground`).
- **Local Persistence:** Room (Database version `121`, 90+ entities) encrypted with **SQLCipher** (`DatabaseKeyManager` stores 256-bit passphrase in `EncryptedSharedPreferences` backed by Android Keystore).
- **Licensing:**
  - Application (`app/`): `GPL-3.0-or-later`
  - Backend (`supabase/`): `AGPL-3.0-or-later`
  - Docs & Specs (`docs/`, `*.md`): `CC BY-SA 4.0`

### 2.2 Modules Identified in Android (`AppScreen`)
1. **Communication:** My Chats (`ChatList`), My Calls (WebRTC audio/video), My SMS/MMS (Android dual-SIM telephony).
2. **Productivity:** My Notes (WYSIWYG/Markdown + audio/attachments), My Calendar (events, tasks, reminders, recurring rules, attendees), My Todo (lists, priorities, reminders), My Habits (schedules, logs, streaks, milestones, grace uses), My Books (EPUB/PDF + Reading XP), My Docs (viewer + search), Drawings.
3. **Security & Privacy:** My Vault (encrypted file storage + folders + tags + trash/archive), My Secrets (password manager, password history, TOTP authenticator, backup codes), Security & Privacy Center, Automatic Backups (`.ktb`).
4. **Finance & Personal:** My Wallet, My Finances (transactions, budgets, goals, assets, debts, recurring, salary/work logs), My E-Mail (IMAP/SMTP, Gmail/Microsoft OAuth), My Feed (RSS/Atom + Readability), My Market (listings/chats + financial tickers), My Browser (privacy browser + Astronomical Sky Engine), My Maps (MapLibre GL + saved places), My Din (prayer schedule, Hijri calendar, Qibla, Quran/Douaa).

### 2.3 Reusable First-Party Assets from Android
- **Official Vector Logo:** `app/src/main/res/drawable/ic_keeftalk_logo.xml` (`1024x1024` viewport, `#0883F3` -> `#3838EC` circular gradient with white K glyph).
- **Sound Effects:** `app/src/main/res/raw/*.mp3` and `sound effects/` (ringtone, ringback, message send/receive/seen, task completion, bell notification).
- **Emoji Dataset:** `app/src/main/assets/emojis.json`.
- **Theme Tokens:** `ui/theme/Color.kt` (Light, Dark, Day, Pinky, Masculine, Brand Cyan `#00CCCC` / Purple `#7D5CFF`) and `ui/theme/keef/KeefDesignTokens.kt` (`DAWN`, `DAY`, `DUSK`, `NIGHT`).
- **Translations:** `res/values/strings.xml`, `values-fr/strings.xml`, `values-ar/strings.xml` (English, French, Arabic).

---

## 3. Backend Reference Repository (`Keeftalk/Backend`)

### 3.1 Verified Infrastructure
- **Platform:** Supabase (PostgreSQL 17, GoTrue Auth, PostgREST, Realtime WebSockets, Deno 2 Edge Functions) + Cloudflare R2 (regional private/public buckets via S3-compatible presigned URLs).
- **Snapshot Verified:** `supabase/snapshots/production_schema_20261125.sql` and modular `supabase/schema/tables/*.sql`.

### 3.2 Verified Edge Functions (`supabase/functions/`)
| Edge Function | Verified Behavior | Desktop Relevance |
|---|---|---|
| `storage` | Generates R2 presigned PUT/GET URLs, runs `HEAD` for `finalize-upload`, manages `move-to-trash`/`restore-from-trash`/`delete-object`, enforces 1 GB free allowance before checking `billing_balances` (`402 PREPAID_BALANCE_EMPTY`). | **Required** for Vault & chat media attachments. |
| `app-sync` | Fetches `profile`, `chats` (15), `calendar_items` (50), `notes` (50), `vault_items` (20). | **Caution:** See schema discrepancy below. |
| `billing-pricing` / `billing-usage` / `billing-invoices` / `billing-admin` | Authoritative pay-as-you-go pricing, 30-day usage estimation, and invoicing. | Used by Settings / Cloud Billing UI. |
| `two-factor-auth` | Twilio SMS OTP dispatch (`send_sms_otp`), returns `501` if Twilio env vars absent. | Optional SMS 2FA; TOTP/backup codes stored in DB. |
| `link-preview` | Fetches OpenGraph metadata for chat URLs. | Chat link previews. |
| `public-profile` / `web-profile` | Public user profile lookups. | Profile discovery. |
| `habit-checkin` / `habit-insights` | Habit check-in & analytics endpoints. | Phase 7 Habits sync. |
| `vault-share-manager` / `vault-share-access` | Controlled sharing of vault items. | Vault sharing. |
| `store-api` / `redeem-subscription-key` / `verify-purchase` / `fcm-dispatcher` | Extension store, key redemption, mobile IAP, FCM push. | `fcm-dispatcher` is mobile-specific; desktop uses Realtime + OS notifications. |

### 3.3 Discrepancies & Gotchas Found During Code Audit
1. **`app-sync` Edge Function vs. `calendar_items` Schema:**
   - `supabase/functions/app-sync/index.ts` line 51 queries `calendar_items` with `.eq('user_id', userId)`, whereas `public.calendar_items` defines the column as `owner_id` (not `user_id`).
   - **Desktop Decision:** Query `calendar_items` directly via PostgREST filtering on `owner_id = auth.uid()` (matching `CalendarRepositoryImpl.kt`), and do not depend blindly on `app-sync` for calendar sync.
2. **Storage Object Key Layout:**
   - `storage/index.ts` prefixes private objects as `{COUNTRY_CODE}/{userId}/{category}/{fileId}/{subpath}` (defaults: `TN` / `WEUR` / `keeftalk-weur-private` when unassigned). Desktop must always obtain `objectKey` from the `storage` Edge Function rather than constructing R2 paths locally.
3. **WebRTC Signaling:**
   - Implemented directly over PostgreSQL tables `public.call_sessions` and `public.call_signaling` + Supabase Realtime `postgres_changes`, using Google STUN and `openrelay.metered.ca` TURN servers (`CallManager.kt`).
4. **SMS/MMS on Desktop:**
   - `public.sms_messages` and `public.sms_threads` exist in Supabase for Android-originated sync, but desktop OSes lack carrier SIM telephony APIs. Desktop will **never fake** carrier SMS sending and will clearly label native telephony as an Android-only platform capability.
5. **Deterministic Fallback PCK in Android (`ConversationKeyManager.kt`):**
   - When no row exists in `conversation_keys`, Android falls back to `HMAC-SHA256("keeftalk-shared-pck-v1", conversationId.lowercase().trim())`. Because `conversationId` is not secret, this fallback offers obfuscation rather than true E2EE against a server adversary. Desktop must support decrypting legacy deterministic-fallback messages for compatibility, while preferring randomly generated CPK-wrapped PCKs and documenting the migration path in `THREAT_MODEL.md`.

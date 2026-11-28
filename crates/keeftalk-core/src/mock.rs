//! Deterministic realistic local demo dataset for `KEEFTALK_MODE=mock`.
//!
//! Mock mode never contacts remote servers and clearly identifies itself in the UI.

use chrono::{Duration, Utc};
use keeftalk_auth::AuthState;
use keeftalk_settings::AppMode;
use keeftalk_sync::{SyncStatus, SyncSummary};

use crate::model::{
    BookItem, BrowserBookmark, CalendarEvent, CallSession, ChatConversation, ChatMessage, DocItem,
    EmailMessage, FinanceTransaction, HabitItem, MapBookmark, NoteItem, PrayerSchedule,
    RssFeedItem, SecretItem, TodoPriority, TodoTask, VaultEntry, WorkspaceSnapshot,
};

pub fn generate_demo_snapshot() -> WorkspaceSnapshot {
    let now = Utc::now();

    let conversations = vec![
        ChatConversation {
            id: "demo-chat-1".into(),
            title: "Amira Benali".into(),
            is_group: false,
            unread_count: 1,
            last_message_preview: "Everything is encrypted end-to-end with PCK v4.".into(),
            last_activity: now - Duration::minutes(4),
            messages: vec![
                ChatMessage {
                    id: "demo-msg-1".into(),
                    chat_id: "demo-chat-1".into(),
                    sender_name: "Amira Benali".into(),
                    is_outgoing: false,
                    body: "Hey! Did the Keeftalk Desktop build finish cleanly?".into(),
                    timestamp: now - Duration::minutes(12),
                    encrypted: true,
                    sync_status: SyncStatus::LocalOnly,
                },
                ChatMessage {
                    id: "demo-msg-2".into(),
                    chat_id: "demo-chat-1".into(),
                    sender_name: "Demo User".into(),
                    is_outgoing: true,
                    body: "Everything is encrypted end-to-end with PCK v4.".into(),
                    timestamp: now - Duration::minutes(4),
                    encrypted: true,
                    sync_status: SyncStatus::LocalOnly,
                },
            ],
        },
        ChatConversation {
            id: "demo-chat-2".into(),
            title: "Keeftalk Contributors".into(),
            is_group: true,
            unread_count: 0,
            last_message_preview: "Welcome to local mock mode — zero cloud account needed."
                .into(),
            last_activity: now - Duration::hours(2),
            messages: vec![ChatMessage {
                id: "demo-msg-3".into(),
                chat_id: "demo-chat-2".into(),
                sender_name: "Maintainer".into(),
                is_outgoing: false,
                body: "Welcome to local mock mode — zero cloud account needed.".into(),
                timestamp: now - Duration::hours(2),
                encrypted: true,
                sync_status: SyncStatus::LocalOnly,
            }],
        },
    ];

    let notes = vec![
        NoteItem {
            id: "demo-note-1".into(),
            title: "Keeftalk Desktop Architecture Notes".into(),
            markdown: "# Zero-Knowledge Offline-First\n\n- Local SQLCipher storage\n- Optional encrypted Supabase + R2 sync\n- No ads, no tracking, no subscriptions".into(),
            tags: vec!["architecture".into(), "rust".into()],
            pinned: true,
            updated_at: now - Duration::minutes(35),
            sync_status: SyncStatus::LocalOnly,
        },
        NoteItem {
            id: "demo-note-2".into(),
            title: "Self-Hosting Checklist".into(),
            markdown: "1. Deploy Supabase project\n2. Provision R2 bucket\n3. Set KEEFTALK_SUPABASE_URL & KEEFTALK_SUPABASE_ANON_KEY".into(),
            tags: vec!["self-hosting".into()],
            pinned: false,
            updated_at: now - Duration::days(1),
            sync_status: SyncStatus::LocalOnly,
        },
    ];

    let todos = vec![
        TodoTask {
            id: "demo-todo-1".into(),
            title: "Verify cross-platform Argon2id + AES-GCM vectors".into(),
            notes: "Must match Android Argon2idManager (64 MiB, t=3, p=4)".into(),
            list_name: "Engineering".into(),
            priority: TodoPriority::High,
            completed: false,
            due_date: Some("2026-11-28".into()),
            sync_status: SyncStatus::LocalOnly,
        },
        TodoTask {
            id: "demo-todo-2".into(),
            title: "Review offline sync queue design".into(),
            notes: "".into(),
            list_name: "Engineering".into(),
            priority: TodoPriority::Medium,
            completed: true,
            due_date: None,
            sync_status: SyncStatus::LocalOnly,
        },
    ];

    let calendar_events = vec![CalendarEvent {
        id: "demo-cal-1".into(),
        title: "Keeftalk Open-Source Community Sync".into(),
        description: "Review MVP milestones and contributor docs.".into(),
        start_time: now + Duration::hours(3),
        end_time: now + Duration::hours(4),
        is_all_day: false,
        location: Some("Keeftalk Encrypted Call".into()),
        sync_status: SyncStatus::LocalOnly,
    }];

    let vault_items = vec![VaultEntry {
        id: "demo-vault-1".into(),
        name: "recovery-backup-sample.ktb".into(),
        folder: "Backups".into(),
        size_bytes: 4096,
        mime_type: "application/octet-stream".into(),
        encrypted: true,
        sync_status: SyncStatus::LocalOnly,
    }];

    let secrets = vec![
        SecretItem {
            id: "sec-1".into(),
            name: "GitHub Developer Key".into(),
            category: "Developer".into(),
            username: "mourad-abidi".into(),
            secret_value: "ghp_xK92mN10vP9qL34zS18".into(),
            totp_secret: Some("JBSWY3DPEHPK3PXP".into()),
            sync_status: SyncStatus::LocalOnly,
        },
        SecretItem {
            id: "sec-2".into(),
            name: "Supabase Service Key".into(),
            category: "Cloud".into(),
            username: "admin@keeftalk.co".into(),
            secret_value: "eyJhbGciOiJIUzI1NiI...".into(),
            totp_secret: None,
            sync_status: SyncStatus::LocalOnly,
        },
    ];

    let habits = vec![
        HabitItem {
            id: "hab-1".into(),
            title: "Read 20 pages of Rust Programming".into(),
            target_frequency: "Daily".into(),
            streak_days: 14,
            completed_today: true,
            sync_status: SyncStatus::LocalOnly,
        },
        HabitItem {
            id: "hab-2".into(),
            title: "30-minute afternoon walk".into(),
            target_frequency: "Daily".into(),
            streak_days: 6,
            completed_today: false,
            sync_status: SyncStatus::LocalOnly,
        },
    ];

    let books = vec![
        BookItem {
            id: "book-1".into(),
            title: "Programming Rust (2nd Edition)".into(),
            author: "Jim Blandy & Jason Orendorff".into(),
            total_pages: 732,
            current_page: 450,
            format: "ePub".into(),
            sync_status: SyncStatus::LocalOnly,
        },
        BookItem {
            id: "book-2".into(),
            title: "Designing Data-Intensive Applications".into(),
            author: "Martin Kleppmann".into(),
            total_pages: 616,
            current_page: 280,
            format: "PDF".into(),
            sync_status: SyncStatus::LocalOnly,
        },
    ];

    let docs = vec![
        DocItem {
            id: "doc-1".into(),
            name: "ADR-001-GUI-FRAMEWORK.md".into(),
            file_type: "Markdown".into(),
            size_bytes: 8192,
            tags: vec!["architecture".into(), "decision".into()],
            updated_at: now - Duration::days(2),
            sync_status: SyncStatus::LocalOnly,
        },
        DocItem {
            id: "doc-2".into(),
            name: "THREAT_MODEL.md".into(),
            file_type: "Markdown".into(),
            size_bytes: 12400,
            tags: vec!["security".into(), "crypto".into()],
            updated_at: now - Duration::days(1),
            sync_status: SyncStatus::LocalOnly,
        },
    ];

    let transactions = vec![
        FinanceTransaction {
            id: "tx-1".into(),
            title: "Cloudflare Domain Renewal".into(),
            amount: 14.99,
            category: "Infrastructure".into(),
            is_expense: true,
            timestamp: now - Duration::days(3),
            sync_status: SyncStatus::LocalOnly,
        },
        FinanceTransaction {
            id: "tx-2".into(),
            title: "Open-Source Sponsor Grant".into(),
            amount: 250.00,
            category: "Income".into(),
            is_expense: false,
            timestamp: now - Duration::days(1),
            sync_status: SyncStatus::LocalOnly,
        },
    ];

    let feeds = vec![
        RssFeedItem {
            id: "rss-1".into(),
            title: "Rust 1.85.0 Released with 2024 Edition Support".into(),
            source_name: "Rust Blog".into(),
            article_url: "https://blog.rust-lang.org/2026/11/20/Rust-1.85.0.html".into(),
            published_at: now - Duration::hours(5),
            is_read: false,
        },
        RssFeedItem {
            id: "rss-2".into(),
            title: "Zero-Knowledge Cryptography in Modern Desktop Apps".into(),
            source_name: "Keeftalk Tech".into(),
            article_url: "https://keeftalk.com/blog/zk-desktop".into(),
            published_at: now - Duration::days(1),
            is_read: true,
        },
    ];

    let bookmarks = vec![
        BrowserBookmark {
            id: "bm-1".into(),
            title: "Keeftalk GitHub Repository".into(),
            url: "https://github.com/Keeftalk/Desktop".into(),
            category: "Development".into(),
        },
        BrowserBookmark {
            id: "bm-2".into(),
            title: "Rust Language Reference".into(),
            url: "https://doc.rust-lang.org/reference/".into(),
            category: "Reference".into(),
        },
    ];

    let emails = vec![
        EmailMessage {
            id: "em-1".into(),
            sender: "notifications@github.com".into(),
            subject: "PR #1 Merged: Project Foundation".into(),
            preview: "Your pull request #1 has been merged into main.".into(),
            timestamp: now - Duration::hours(1),
            is_unread: true,
        },
        EmailMessage {
            id: "em-2".into(),
            sender: "security@keeftalk.co".into(),
            subject: "Zero-Knowledge Key Backup Confirmed".into(),
            preview: "Your AEK verification tag was validated successfully.".into(),
            timestamp: now - Duration::days(1),
            is_unread: false,
        },
    ];

    let map_bookmarks = vec![
        MapBookmark {
            id: "map-1".into(),
            title: "Keeftalk Innovation Lab".into(),
            address: "Avenue Habib Bourguiba, Tunis".into(),
            latitude: 36.8065,
            longitude: 10.1815,
            category: "Work".into(),
        },
        MapBookmark {
            id: "map-2".into(),
            title: "Al-Zaytuna Mosque".into(),
            address: "Medina of Tunis".into(),
            latitude: 36.7983,
            longitude: 10.1711,
            category: "Personal".into(),
        },
    ];

    let prayer_schedule = PrayerSchedule {
        fajr: "05:12".into(),
        dhuhr: "12:30".into(),
        asr: "15:45".into(),
        maghrib: "18:15".into(),
        isha: "19:42".into(),
        next_prayer: "Maghrib".into(),
        minutes_remaining: 25,
    };

    let call_session = Some(CallSession {
        id: "call-1".into(),
        peer_name: "Amira Benali".into(),
        is_video: true,
        active: false,
        duration_secs: 145,
        is_muted: false,
    });

    WorkspaceSnapshot {
        mode: AppMode::Mock,
        auth: AuthState::MockDemo {
            user_id: "00000000-0000-4000-8000-00000000demo".into(),
            display_name: "Demo Contributor".into(),
        },
        auth_form: Default::default(),
        sync: SyncSummary {
            status: SyncStatus::LocalOnly,
            pending_items: 0,
            last_synced_at: None,
            last_error: Some("Mock/Demo mode active (cloud sync disabled)".into()),
        },
        conversations,
        notes,
        todos,
        calendar_events,
        vault_items,
        secrets,
        habits,
        books,
        docs,
        transactions,
        feeds,
        bookmarks,
        emails,
        map_bookmarks,
        prayer_schedule,
        call_session,
    }
}

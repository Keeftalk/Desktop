//! Deterministic realistic local demo dataset for `KEEFTALK_MODE=mock`.
//!
//! Mock mode never contacts remote servers and clearly identifies itself in the UI.

use chrono::{Duration, Utc};
use keeftalk_auth::AuthState;
use keeftalk_settings::AppMode;
use keeftalk_sync::{SyncStatus, SyncSummary};

use crate::model::{
    CalendarEvent, ChatConversation, ChatMessage, NoteItem, TodoPriority, TodoTask, VaultEntry,
    WorkspaceSnapshot,
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

    WorkspaceSnapshot {
        mode: AppMode::Mock,
        auth: AuthState::MockDemo {
            user_id: "00000000-0000-4000-8000-00000000demo".into(),
            display_name: "Demo Contributor".into(),
        },
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
    }
}

//! SQLCipher-backed, E2EE-enabled repositories for Keeftalk Desktop MVP modules.

use chrono::{DateTime, TimeZone, Utc};
use keeftalk_crypto::messaging::{ENVELOPE_TYPE_PCK, MESSAGE_CRYPTO_VERSION, encrypt_message_v4};
use keeftalk_crypto::{AccountKeyContext, Key256, aead};
use keeftalk_database::{DatabaseError, EncryptedDatabase};
use keeftalk_sync::{OutboxStore, SyncStatus};
use uuid::Uuid;

use crate::CoreError;
use crate::model::{
    CalendarEvent, ChatConversation, ChatMessage, NoteItem, TodoPriority, TodoTask, VaultEntry,
};

pub struct LocalRepository {
    db: EncryptedDatabase,
    outbox: OutboxStore,
    aek: AccountKeyContext,
    user_id: String,
    queue_remote_sync: bool,
}

impl LocalRepository {
    pub fn new(
        db: EncryptedDatabase,
        aek: AccountKeyContext,
        user_id: impl Into<String>,
        queue_remote_sync: bool,
    ) -> Self {
        let outbox = OutboxStore::new(db.clone());
        Self {
            db,
            outbox,
            aek,
            user_id: user_id.into(),
            queue_remote_sync,
        }
    }

    pub fn db(&self) -> &EncryptedDatabase {
        &self.db
    }

    pub fn pending_sync_count(&self) -> Result<usize, CoreError> {
        Ok(self.outbox.pending_count()?)
    }

    pub fn seed_demo_if_empty(&self, demo: &crate::model::WorkspaceSnapshot) -> Result<(), CoreError> {
        let existing_chats: i64 = self.db.with_conn(|c| {
            c.query_row("SELECT count(*) FROM chats", [], |r| r.get(0))
                .map_err(DatabaseError::from)
        })?;
        if existing_chats > 0 {
            return Ok(());
        }

        for conv in &demo.conversations {
            self.db.with_conn(|c| {
                c.execute(
                    "INSERT INTO chats (id, title, is_group, unread_count, last_message_preview, last_activity_ms, sync_status)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    (
                        &conv.id,
                        &conv.title,
                        conv.is_group as i64,
                        conv.unread_count as i64,
                        &conv.last_message_preview,
                        conv.last_activity.timestamp_millis(),
                        "LocalOnly",
                    ),
                )?;
                for m in &conv.messages {
                    c.execute(
                        "INSERT INTO messages (id, chat_id, sender_id, sender_name, is_outgoing, content, timestamp_ms, sync_status)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                        (
                            &m.id,
                            &m.chat_id,
                            if m.is_outgoing { &self.user_id } else { "peer" },
                            &m.sender_name,
                            m.is_outgoing as i64,
                            &m.body,
                            m.timestamp.timestamp_millis(),
                            "LocalOnly",
                        ),
                    )?;
                }
                Ok(())
            })?;
        }

        for note in &demo.notes {
            self.save_note(
                Some(note.id.clone()),
                &note.title,
                &note.markdown,
                note.tags.clone(),
                note.pinned,
            )?;
        }

        for task in &demo.todos {
            self.create_todo_with_id(
                task.id.clone(),
                &task.title,
                &task.notes,
                &task.list_name,
                task.priority,
                task.completed,
                task.due_date.clone(),
            )?;
        }

        for ev in &demo.calendar_events {
            self.create_event_with_id(
                ev.id.clone(),
                &ev.title,
                &ev.description,
                ev.start_time,
                ev.end_time,
                ev.is_all_day,
                ev.location.clone(),
            )?;
        }

        for v in &demo.vault_items {
            self.add_vault_entry_with_id(
                v.id.clone(),
                &v.name,
                &v.folder,
                v.size_bytes,
                &v.mime_type,
            )?;
        }

        Ok(())
    }

    // --- Chats ---

    pub fn list_conversations(&self) -> Result<Vec<ChatConversation>, CoreError> {
        self.db
            .with_conn(|c| {
                let mut stmt = c.prepare(
                    "SELECT id, title, is_group, unread_count, last_message_preview, last_activity_ms
                     FROM chats ORDER BY last_activity_ms DESC",
                )?;
                let mut convs = stmt
                    .query_map([], |r| {
                        let ts_ms: i64 = r.get(5)?;
                        Ok(ChatConversation {
                            id: r.get(0)?,
                            title: r.get(1)?,
                            is_group: r.get::<_, i64>(2)? != 0,
                            unread_count: r.get::<_, i64>(3)? as u32,
                            last_message_preview: r.get(4)?,
                            last_activity: ms_to_utc(ts_ms),
                            messages: Vec::new(),
                        })
                    })?
                    .collect::<Result<Vec<_>, _>>()?;

                let mut msg_stmt = c.prepare(
                    "SELECT id, chat_id, sender_name, is_outgoing, content, timestamp_ms, ciphertext, sync_status
                     FROM messages WHERE chat_id = ?1 ORDER BY timestamp_ms ASC",
                )?;
                for conv in &mut convs {
                    let msgs = msg_stmt
                        .query_map([&conv.id], |r| {
                            let ts_ms: i64 = r.get(5)?;
                            let cipher: Option<String> = r.get(6)?;
                            let sync_str: String = r.get(7)?;
                            Ok(ChatMessage {
                                id: r.get(0)?,
                                chat_id: r.get(1)?,
                                sender_name: r.get(2)?,
                                is_outgoing: r.get::<_, i64>(3)? != 0,
                                body: r.get(4)?,
                                timestamp: ms_to_utc(ts_ms),
                                encrypted: cipher.is_some(),
                                sync_status: parse_sync_status(&sync_str),
                            })
                        })?
                        .collect::<Result<Vec<_>, _>>()?;
                    conv.messages = msgs;
                }
                Ok(convs)
            })
            .map_err(CoreError::from)
    }

    pub fn send_message(&self, chat_id: &str, body: &str) -> Result<ChatMessage, CoreError> {
        let msg_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let now_ms = now.timestamp_millis();

        // Derive/obtain PCK and encrypt payload with PCK v4 AAD
        let pck = Key256::random();
        let env = encrypt_message_v4(&pck, chat_id, &msg_id, body)?;
        let sync_status = if self.queue_remote_sync {
            SyncStatus::PendingUpload
        } else {
            SyncStatus::LocalOnly
        };
        let sync_str = sync_status_str(sync_status);

        self.db.with_conn(|c| {
            c.execute(
                "INSERT INTO messages (id, chat_id, sender_id, sender_name, is_outgoing, content, ciphertext, nonce, crypto_version, envelope_type, timestamp_ms, sync_status)
                 VALUES (?1, ?2, ?3, 'You', 1, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                (
                    &msg_id,
                    chat_id,
                    &self.user_id,
                    body,
                    &env.ciphertext_b64,
                    &env.nonce_b64,
                    MESSAGE_CRYPTO_VERSION as i64,
                    ENVELOPE_TYPE_PCK as i64,
                    now_ms,
                    sync_str,
                ),
            )?;
            c.execute(
                "UPDATE chats SET last_message_preview = ?2, last_activity_ms = ?3 WHERE id = ?1",
                (chat_id, body, now_ms),
            )?;
            Ok(())
        })?;

        if self.queue_remote_sync {
            self.outbox.enqueue(
                "message",
                &msg_id,
                "UPSERT",
                &serde_json::json!({
                    "id": msg_id,
                    "chat_id": chat_id,
                    "sender_id": self.user_id,
                    "content": "[Encrypted]",
                    "ciphertext": env.ciphertext_b64,
                    "nonce": env.nonce_b64,
                    "crypto_version": MESSAGE_CRYPTO_VERSION,
                    "envelope_type": ENVELOPE_TYPE_PCK,
                    "timestamp": now_ms,
                }),
            )?;
        }

        Ok(ChatMessage {
            id: msg_id,
            chat_id: chat_id.to_string(),
            sender_name: "You".into(),
            is_outgoing: true,
            body: body.to_string(),
            timestamp: now,
            encrypted: true,
            sync_status,
        })
    }

    // --- Notes ---

    pub fn list_notes(&self) -> Result<Vec<NoteItem>, CoreError> {
        self.db
            .with_conn(|c| {
                let mut stmt = c.prepare(
                    "SELECT id, title, markdown, tags_json, pinned, updated_at_ms, sync_status
                     FROM notes WHERE archived = 0 ORDER BY pinned DESC, updated_at_ms DESC",
                )?;
                let rows = stmt
                    .query_map([], |r| {
                        let tags_raw: String = r.get(3)?;
                        let tags: Vec<String> = serde_json::from_str(&tags_raw).unwrap_or_default();
                        let sync_str: String = r.get(6)?;
                        Ok(NoteItem {
                            id: r.get(0)?,
                            title: r.get(1)?,
                            markdown: r.get(2)?,
                            tags,
                            pinned: r.get::<_, i64>(4)? != 0,
                            updated_at: ms_to_utc(r.get(5)?),
                            sync_status: parse_sync_status(&sync_str),
                        })
                    })?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(rows)
            })
            .map_err(CoreError::from)
    }

    pub fn save_note(
        &self,
        id: Option<String>,
        title: &str,
        markdown: &str,
        tags: Vec<String>,
        pinned: bool,
    ) -> Result<NoteItem, CoreError> {
        let note_id = id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let now = Utc::now();
        let now_ms = now.timestamp_millis();
        let tags_json = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string());

        let notes_key = self.aek.notes_key()?;
        let sensitive = serde_json::json!({ "title": title, "content": markdown }).to_string();
        let encrypted = aead::encrypt(sensitive.as_bytes(), &notes_key, None)?;

        let sync_status = if self.queue_remote_sync {
            SyncStatus::PendingUpload
        } else {
            SyncStatus::LocalOnly
        };

        self.db.with_conn(|c| {
            c.execute(
                "INSERT INTO notes (id, owner_id, title, markdown, tags_json, pinned, ciphertext, iv, crypto_version, updated_at_ms, sync_status)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                 ON CONFLICT(id) DO UPDATE SET
                    title = excluded.title,
                    markdown = excluded.markdown,
                    tags_json = excluded.tags_json,
                    pinned = excluded.pinned,
                    ciphertext = excluded.ciphertext,
                    iv = excluded.iv,
                    updated_at_ms = excluded.updated_at_ms,
                    sync_status = excluded.sync_status",
                (
                    &note_id,
                    &self.user_id,
                    title,
                    markdown,
                    &tags_json,
                    pinned as i64,
                    &encrypted.ciphertext,
                    &encrypted.iv,
                    encrypted.version as i64,
                    now_ms,
                    sync_status_str(sync_status),
                ),
            )?;
            Ok(())
        })?;

        if self.queue_remote_sync {
            self.outbox.enqueue(
                "note",
                &note_id,
                "UPSERT",
                &serde_json::json!({
                    "id": note_id,
                    "owner_id": self.user_id,
                    "title": "[Encrypted]",
                    "content": "",
                    "pinned": pinned,
                    "ciphertext": encrypted.ciphertext,
                    "iv": encrypted.iv,
                    "crypto_version": encrypted.version,
                }),
            )?;
        }

        Ok(NoteItem {
            id: note_id,
            title: title.to_string(),
            markdown: markdown.to_string(),
            tags,
            pinned,
            updated_at: now,
            sync_status,
        })
    }

    // --- Todo ---

    pub fn list_todos(&self) -> Result<Vec<TodoTask>, CoreError> {
        self.db
            .with_conn(|c| {
                let mut stmt = c.prepare(
                    "SELECT id, title, notes, COALESCE(list_id, 'General'), priority, completed, due_date, sync_status
                     FROM todo_tasks WHERE deleted = 0 ORDER BY completed ASC, priority DESC, created_at_ms DESC",
                )?;
                let rows = stmt
                    .query_map([], |r| {
                        let prio = match r.get::<_, i64>(4)? {
                            0 => TodoPriority::Low,
                            1 => TodoPriority::Medium,
                            2 => TodoPriority::High,
                            _ => TodoPriority::Urgent,
                        };
                        let sync_str: String = r.get(7)?;
                        Ok(TodoTask {
                            id: r.get(0)?,
                            title: r.get(1)?,
                            notes: r.get(2)?,
                            list_name: r.get(3)?,
                            priority: prio,
                            completed: r.get::<_, i64>(5)? != 0,
                            due_date: r.get(6)?,
                            sync_status: parse_sync_status(&sync_str),
                        })
                    })?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(rows)
            })
            .map_err(CoreError::from)
    }

    pub fn create_todo(
        &self,
        title: &str,
        list_name: &str,
        priority: TodoPriority,
        due_date: Option<String>,
    ) -> Result<TodoTask, CoreError> {
        self.create_todo_with_id(
            Uuid::new_v4().to_string(),
            title,
            "",
            list_name,
            priority,
            false,
            due_date,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn create_todo_with_id(
        &self,
        id: String,
        title: &str,
        notes: &str,
        list_name: &str,
        priority: TodoPriority,
        completed: bool,
        due_date: Option<String>,
    ) -> Result<TodoTask, CoreError> {
        let now_ms = Utc::now().timestamp_millis();
        let prio_num = match priority {
            TodoPriority::Low => 0,
            TodoPriority::Medium => 1,
            TodoPriority::High => 2,
            TodoPriority::Urgent => 3,
        };
        let sync_status = if self.queue_remote_sync {
            SyncStatus::PendingUpload
        } else {
            SyncStatus::LocalOnly
        };

        self.db.with_conn(|c| {
            c.execute(
                "INSERT INTO todo_tasks (id, user_id, list_id, title, notes, due_date, priority, completed, created_at_ms, updated_at_ms, deleted, sync_status)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9, 0, ?10)",
                (
                    &id,
                    &self.user_id,
                    list_name,
                    title,
                    notes,
                    due_date.as_deref(),
                    prio_num,
                    completed as i64,
                    now_ms,
                    sync_status_str(sync_status),
                ),
            )?;
            Ok(())
        })?;

        Ok(TodoTask {
            id,
            title: title.to_string(),
            notes: notes.to_string(),
            list_name: list_name.to_string(),
            priority,
            completed,
            due_date,
            sync_status,
        })
    }

    pub fn toggle_todo(&self, id: &str, completed: bool) -> Result<(), CoreError> {
        let now_ms = Utc::now().timestamp_millis();
        self.db
            .with_conn(|c| {
                c.execute(
                    "UPDATE todo_tasks SET completed = ?2, updated_at_ms = ?3 WHERE id = ?1",
                    (id, completed as i64, now_ms),
                )?;
                Ok(())
            })
            .map_err(CoreError::from)
    }

    // --- Calendar ---

    pub fn list_events(&self) -> Result<Vec<CalendarEvent>, CoreError> {
        self.db
            .with_conn(|c| {
                let mut stmt = c.prepare(
                    "SELECT id, title, description, start_time_ms, end_time_ms, is_all_day, location, sync_status
                     FROM calendar_items ORDER BY start_time_ms ASC",
                )?;
                let rows = stmt
                    .query_map([], |r| {
                        let sync_str: String = r.get(7)?;
                        Ok(CalendarEvent {
                            id: r.get(0)?,
                            title: r.get(1)?,
                            description: r.get(2)?,
                            start_time: ms_to_utc(r.get(3)?),
                            end_time: ms_to_utc(r.get(4)?),
                            is_all_day: r.get::<_, i64>(5)? != 0,
                            location: r.get(6)?,
                            sync_status: parse_sync_status(&sync_str),
                        })
                    })?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(rows)
            })
            .map_err(CoreError::from)
    }

    #[allow(clippy::too_many_arguments)]
    fn create_event_with_id(
        &self,
        id: String,
        title: &str,
        description: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        is_all_day: bool,
        location: Option<String>,
    ) -> Result<CalendarEvent, CoreError> {
        let agenda_key = self.aek.agenda_key()?;
        let sensitive = serde_json::json!({ "title": title, "description": description }).to_string();
        let encrypted = aead::encrypt(sensitive.as_bytes(), &agenda_key, None)?;
        let now_ms = Utc::now().timestamp_millis();
        let sync_status = if self.queue_remote_sync {
            SyncStatus::PendingUpload
        } else {
            SyncStatus::LocalOnly
        };

        self.db.with_conn(|c| {
            c.execute(
                "INSERT INTO calendar_items (id, owner_id, title, description, start_time_ms, end_time_ms, is_all_day, location, ciphertext, iv, crypto_version, updated_at_ms, sync_status)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                (
                    &id,
                    &self.user_id,
                    title,
                    description,
                    start_time.timestamp_millis(),
                    end_time.timestamp_millis(),
                    is_all_day as i64,
                    location.as_deref(),
                    &encrypted.ciphertext,
                    &encrypted.iv,
                    encrypted.version as i64,
                    now_ms,
                    sync_status_str(sync_status),
                ),
            )?;
            Ok(())
        })?;

        Ok(CalendarEvent {
            id,
            title: title.to_string(),
            description: description.to_string(),
            start_time,
            end_time,
            is_all_day,
            location,
            sync_status,
        })
    }

    // --- Vault ---

    pub fn list_vault_items(&self) -> Result<Vec<VaultEntry>, CoreError> {
        self.db
            .with_conn(|c| {
                let mut stmt = c.prepare(
                    "SELECT id, title, folder, size_bytes, mime_type, sync_status
                     FROM vault_items WHERE is_deleted = 0 ORDER BY created_at_ms DESC",
                )?;
                let rows = stmt
                    .query_map([], |r| {
                        let sync_str: String = r.get(5)?;
                        Ok(VaultEntry {
                            id: r.get(0)?,
                            name: r.get(1)?,
                            folder: r.get(2)?,
                            size_bytes: r.get::<_, i64>(3)? as u64,
                            mime_type: r.get(4)?,
                            encrypted: true,
                            sync_status: parse_sync_status(&sync_str),
                        })
                    })?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(rows)
            })
            .map_err(CoreError::from)
    }

    fn add_vault_entry_with_id(
        &self,
        id: String,
        name: &str,
        folder: &str,
        size_bytes: u64,
        mime_type: &str,
    ) -> Result<VaultEntry, CoreError> {
        let now_ms = Utc::now().timestamp_millis();
        let file_id = Uuid::new_v4().to_string();
        let sync_status = if self.queue_remote_sync {
            SyncStatus::PendingUpload
        } else {
            SyncStatus::LocalOnly
        };

        self.db.with_conn(|c| {
            c.execute(
                "INSERT INTO vault_items (id, user_id, file_id, folder, title, size_bytes, mime_type, created_at_ms, sync_status)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                (
                    &id,
                    &self.user_id,
                    &file_id,
                    folder,
                    name,
                    size_bytes as i64,
                    mime_type,
                    now_ms,
                    sync_status_str(sync_status),
                ),
            )?;
            Ok(())
        })?;

        Ok(VaultEntry {
            id,
            name: name.to_string(),
            folder: folder.to_string(),
            size_bytes,
            mime_type: mime_type.to_string(),
            encrypted: true,
            sync_status,
        })
    }
}

fn ms_to_utc(ms: i64) -> DateTime<Utc> {
    Utc.timestamp_millis_opt(ms)
        .single()
        .unwrap_or_else(Utc::now)
}

fn sync_status_str(s: SyncStatus) -> &'static str {
    match s {
        SyncStatus::LocalOnly => "LocalOnly",
        SyncStatus::PendingUpload => "PendingUpload",
        SyncStatus::Synchronizing => "Synchronizing",
        SyncStatus::Synchronized => "Synchronized",
        SyncStatus::ConflictDetected => "ConflictDetected",
        SyncStatus::SyncFailed => "SyncFailed",
    }
}

fn parse_sync_status(s: &str) -> SyncStatus {
    match s {
        "PendingUpload" => SyncStatus::PendingUpload,
        "Synchronizing" => SyncStatus::Synchronizing,
        "Synchronized" => SyncStatus::Synchronized,
        "ConflictDetected" => SyncStatus::ConflictDetected,
        "SyncFailed" => SyncStatus::SyncFailed,
        _ => SyncStatus::LocalOnly,
    }
}

-- Keeftalk Desktop Initial Encrypted Schema (v1)

CREATE TABLE IF NOT EXISTS chats (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    is_group INTEGER NOT NULL DEFAULT 0,
    unread_count INTEGER NOT NULL DEFAULT 0,
    last_message_preview TEXT NOT NULL DEFAULT '',
    last_activity_ms INTEGER NOT NULL DEFAULT 0,
    sync_status TEXT NOT NULL DEFAULT 'LocalOnly'
);

CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY NOT NULL,
    chat_id TEXT NOT NULL,
    sender_id TEXT NOT NULL,
    sender_name TEXT NOT NULL DEFAULT '',
    is_outgoing INTEGER NOT NULL DEFAULT 0,
    content TEXT NOT NULL DEFAULT '',
    ciphertext TEXT,
    nonce TEXT,
    crypto_version INTEGER NOT NULL DEFAULT 4,
    envelope_type INTEGER NOT NULL DEFAULT 100,
    timestamp_ms INTEGER NOT NULL,
    sync_status TEXT NOT NULL DEFAULT 'LocalOnly',
    FOREIGN KEY (chat_id) REFERENCES chats(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_messages_chat_time ON messages(chat_id, timestamp_ms);

CREATE TABLE IF NOT EXISTS conversation_keys (
    id TEXT PRIMARY KEY NOT NULL,
    conversation_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    encrypted_key TEXT NOT NULL,
    nonce TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 2,
    epoch INTEGER NOT NULL DEFAULT 1
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_conv_keys_unique ON conversation_keys(conversation_id, user_id, epoch);

CREATE TABLE IF NOT EXISTS notes (
    id TEXT PRIMARY KEY NOT NULL,
    owner_id TEXT NOT NULL,
    title TEXT NOT NULL DEFAULT '',
    markdown TEXT NOT NULL DEFAULT '',
    tags_json TEXT NOT NULL DEFAULT '[]',
    pinned INTEGER NOT NULL DEFAULT 0,
    archived INTEGER NOT NULL DEFAULT 0,
    ciphertext TEXT,
    iv TEXT,
    crypto_version INTEGER NOT NULL DEFAULT 1,
    updated_at_ms INTEGER NOT NULL,
    sync_status TEXT NOT NULL DEFAULT 'LocalOnly'
);

CREATE TABLE IF NOT EXISTS todo_lists (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    name TEXT NOT NULL,
    color_hex TEXT NOT NULL DEFAULT '#6C63FF',
    updated_at_ms INTEGER NOT NULL,
    deleted INTEGER NOT NULL DEFAULT 0,
    sync_status TEXT NOT NULL DEFAULT 'LocalOnly'
);

CREATE TABLE IF NOT EXISTS todo_tasks (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    list_id TEXT,
    title TEXT NOT NULL,
    notes TEXT NOT NULL DEFAULT '',
    due_date TEXT,
    priority INTEGER NOT NULL DEFAULT 1,
    completed INTEGER NOT NULL DEFAULT 0,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    deleted INTEGER NOT NULL DEFAULT 0,
    sync_status TEXT NOT NULL DEFAULT 'LocalOnly'
);

CREATE TABLE IF NOT EXISTS calendar_items (
    id TEXT PRIMARY KEY NOT NULL,
    owner_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    start_time_ms INTEGER NOT NULL,
    end_time_ms INTEGER NOT NULL,
    is_all_day INTEGER NOT NULL DEFAULT 0,
    location TEXT,
    recurrence_rule TEXT,
    ciphertext TEXT,
    iv TEXT,
    crypto_version INTEGER NOT NULL DEFAULT 1,
    updated_at_ms INTEGER NOT NULL,
    sync_status TEXT NOT NULL DEFAULT 'LocalOnly'
);

CREATE TABLE IF NOT EXISTS vault_items (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    file_id TEXT NOT NULL,
    folder TEXT NOT NULL DEFAULT 'Root',
    title TEXT NOT NULL,
    size_bytes INTEGER NOT NULL DEFAULT 0,
    mime_type TEXT NOT NULL DEFAULT 'application/octet-stream',
    local_path TEXT,
    storage_path TEXT,
    encryption_metadata TEXT,
    is_deleted INTEGER NOT NULL DEFAULT 0,
    created_at_ms INTEGER NOT NULL,
    sync_status TEXT NOT NULL DEFAULT 'LocalOnly'
);

CREATE TABLE IF NOT EXISTS sync_outbox (
    id TEXT PRIMARY KEY NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    operation TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    next_retry_at_ms INTEGER NOT NULL DEFAULT 0,
    created_at_ms INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_sync_outbox_retry ON sync_outbox(next_retry_at_ms, created_at_ms);

//! Domain models shared across Keeftalk Desktop modules.

use chrono::{DateTime, Utc};
use keeftalk_auth::AuthState;
use keeftalk_settings::AppMode;
use keeftalk_sync::{SyncStatus, SyncSummary};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModuleGroup {
    Communication,
    Productivity,
    Security,
    Finance,
    Discover,
    Personal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModuleId {
    Home,
    Chats,
    Calls,
    SmsInfo,
    Notes,
    Calendar,
    Todo,
    Habits,
    Books,
    Docs,
    Vault,
    Secrets,
    SecurityCenter,
    Wallet,
    Finances,
    Email,
    Feed,
    Market,
    Browser,
    Maps,
    Din,
    Settings,
}

impl ModuleId {
    pub fn title(self) -> &'static str {
        match self {
            Self::Home => "Home",
            Self::Chats => "My Chats",
            Self::Calls => "My Calls",
            Self::SmsInfo => "SMS/MMS",
            Self::Notes => "My Notes",
            Self::Calendar => "My Calendar",
            Self::Todo => "My Todo",
            Self::Habits => "My Habits",
            Self::Books => "My Books",
            Self::Docs => "My Docs",
            Self::Vault => "My Vault",
            Self::Secrets => "My Secrets",
            Self::SecurityCenter => "Security Center",
            Self::Wallet => "My Wallet",
            Self::Finances => "My Finances",
            Self::Email => "My E-Mail",
            Self::Feed => "My Feed",
            Self::Market => "My Market",
            Self::Browser => "My Browser",
            Self::Maps => "My Maps",
            Self::Din => "My Din",
            Self::Settings => "Settings",
        }
    }

    pub fn group(self) -> Option<ModuleGroup> {
        match self {
            Self::Chats | Self::Calls | Self::SmsInfo => Some(ModuleGroup::Communication),
            Self::Notes | Self::Calendar | Self::Todo | Self::Habits | Self::Books | Self::Docs => {
                Some(ModuleGroup::Productivity)
            }
            Self::Vault | Self::Secrets | Self::SecurityCenter => Some(ModuleGroup::Security),
            Self::Wallet | Self::Finances | Self::Market => Some(ModuleGroup::Finance),
            Self::Feed | Self::Browser => Some(ModuleGroup::Discover),
            Self::Email | Self::Maps | Self::Din => Some(ModuleGroup::Personal),
            Self::Home | Self::Settings => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub chat_id: String,
    pub sender_name: String,
    pub is_outgoing: bool,
    pub body: String,
    pub timestamp: DateTime<Utc>,
    pub encrypted: bool,
    pub sync_status: SyncStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatConversation {
    pub id: String,
    pub title: String,
    pub is_group: bool,
    pub unread_count: u32,
    pub last_message_preview: String,
    pub last_activity: DateTime<Utc>,
    pub messages: Vec<ChatMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteItem {
    pub id: String,
    pub title: String,
    pub markdown: String,
    pub tags: Vec<String>,
    pub pinned: bool,
    pub updated_at: DateTime<Utc>,
    pub sync_status: SyncStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TodoPriority {
    Low,
    Medium,
    High,
    Urgent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoTask {
    pub id: String,
    pub title: String,
    pub notes: String,
    pub list_name: String,
    pub priority: TodoPriority,
    pub completed: bool,
    pub due_date: Option<String>,
    pub sync_status: SyncStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: String,
    pub title: String,
    pub description: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub is_all_day: bool,
    pub location: Option<String>,
    pub sync_status: SyncStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultEntry {
    pub id: String,
    pub name: String,
    pub folder: String,
    pub size_bytes: u64,
    pub mime_type: String,
    pub encrypted: bool,
    pub sync_status: SyncStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSnapshot {
    pub mode: AppMode,
    pub auth: AuthState,
    pub sync: SyncSummary,
    pub conversations: Vec<ChatConversation>,
    pub notes: Vec<NoteItem>,
    pub todos: Vec<TodoTask>,
    pub calendar_events: Vec<CalendarEvent>,
    pub vault_items: Vec<VaultEntry>,
}

impl WorkspaceSnapshot {
    pub fn empty(auth: AuthState, mode: AppMode) -> Self {
        Self {
            mode,
            auth,
            sync: SyncSummary::default(),
            conversations: Vec::new(),
            notes: Vec::new(),
            todos: Vec::new(),
            calendar_events: Vec::new(),
            vault_items: Vec::new(),
        }
    }
}

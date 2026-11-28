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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthFormState {
    pub sign_in_email: String,
    pub sign_in_password: String,
    pub master_password: String,
    pub error_message: Option<String>,
    pub is_submitting: bool,
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
pub struct SecretItem {
    pub id: String,
    pub name: String,
    pub category: String,
    pub username: String,
    pub secret_value: String,
    pub totp_secret: Option<String>,
    pub sync_status: SyncStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HabitItem {
    pub id: String,
    pub title: String,
    pub target_frequency: String,
    pub streak_days: u32,
    pub completed_today: bool,
    pub sync_status: SyncStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookItem {
    pub id: String,
    pub title: String,
    pub author: String,
    pub total_pages: u32,
    pub current_page: u32,
    pub format: String,
    pub sync_status: SyncStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocItem {
    pub id: String,
    pub name: String,
    pub file_type: String,
    pub size_bytes: u64,
    pub tags: Vec<String>,
    pub updated_at: DateTime<Utc>,
    pub sync_status: SyncStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinanceTransaction {
    pub id: String,
    pub title: String,
    pub amount: f64,
    pub category: String,
    pub is_expense: bool,
    pub timestamp: DateTime<Utc>,
    pub sync_status: SyncStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RssFeedItem {
    pub id: String,
    pub title: String,
    pub source_name: String,
    pub article_url: String,
    pub published_at: DateTime<Utc>,
    pub is_read: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserBookmark {
    pub id: String,
    pub title: String,
    pub url: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMessage {
    pub id: String,
    pub sender: String,
    pub subject: String,
    pub preview: String,
    pub timestamp: DateTime<Utc>,
    pub is_unread: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapBookmark {
    pub id: String,
    pub title: String,
    pub address: String,
    pub latitude: f64,
    pub longitude: f64,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrayerSchedule {
    pub fajr: String,
    pub dhuhr: String,
    pub asr: String,
    pub maghrib: String,
    pub isha: String,
    pub next_prayer: String,
    pub minutes_remaining: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallSession {
    pub id: String,
    pub peer_name: String,
    pub is_video: bool,
    pub active: bool,
    pub duration_secs: u32,
    pub is_muted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSnapshot {
    pub mode: AppMode,
    pub auth: AuthState,
    pub auth_form: AuthFormState,
    pub sync: SyncSummary,
    pub conversations: Vec<ChatConversation>,
    pub notes: Vec<NoteItem>,
    pub todos: Vec<TodoTask>,
    pub calendar_events: Vec<CalendarEvent>,
    pub vault_items: Vec<VaultEntry>,
    pub secrets: Vec<SecretItem>,
    pub habits: Vec<HabitItem>,
    pub books: Vec<BookItem>,
    pub docs: Vec<DocItem>,
    pub transactions: Vec<FinanceTransaction>,
    pub feeds: Vec<RssFeedItem>,
    pub bookmarks: Vec<BrowserBookmark>,
    pub emails: Vec<EmailMessage>,
    pub map_bookmarks: Vec<MapBookmark>,
    pub prayer_schedule: PrayerSchedule,
    pub call_session: Option<CallSession>,
}

impl WorkspaceSnapshot {
    pub fn empty(auth: AuthState, mode: AppMode) -> Self {
        Self {
            mode,
            auth,
            auth_form: AuthFormState::default(),
            sync: SyncSummary::default(),
            conversations: Vec::new(),
            notes: Vec::new(),
            todos: Vec::new(),
            calendar_events: Vec::new(),
            vault_items: Vec::new(),
            secrets: Vec::new(),
            habits: Vec::new(),
            books: Vec::new(),
            docs: Vec::new(),
            transactions: Vec::new(),
            feeds: Vec::new(),
            bookmarks: Vec::new(),
            emails: Vec::new(),
            map_bookmarks: Vec::new(),
            prayer_schedule: PrayerSchedule {
                fajr: "05:12".into(),
                dhuhr: "12:30".into(),
                asr: "15:45".into(),
                maghrib: "18:15".into(),
                isha: "19:42".into(),
                next_prayer: "Fajr".into(),
                minutes_remaining: 180,
            },
            call_session: None,
        }
    }
}

//! Global search and command palette indexing for Keeftalk Desktop.

use keeftalk_core::model::{ModuleId, WorkspaceSnapshot};
use keeftalk_settings::ThemeVariant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaletteAction {
    Navigate(ModuleId),
    SetTheme(ThemeVariant),
    ToggleSidebar,
    ToggleContextPanel,
}

#[derive(Debug, Clone)]
pub struct PaletteEntry {
    pub title: String,
    pub subtitle: String,
    pub action: PaletteAction,
}

pub fn search_palette(query: &str, snapshot: &WorkspaceSnapshot) -> Vec<PaletteEntry> {
    let q = query.trim().to_ascii_lowercase();
    let mut entries = Vec::new();

    let modules = [
        ModuleId::Home,
        ModuleId::Chats,
        ModuleId::Calls,
        ModuleId::Notes,
        ModuleId::Calendar,
        ModuleId::Todo,
        ModuleId::Habits,
        ModuleId::Books,
        ModuleId::Docs,
        ModuleId::Vault,
        ModuleId::Secrets,
        ModuleId::SecurityCenter,
        ModuleId::Wallet,
        ModuleId::Finances,
        ModuleId::Market,
        ModuleId::Feed,
        ModuleId::Browser,
        ModuleId::Email,
        ModuleId::Maps,
        ModuleId::Din,
        ModuleId::SmsInfo,
        ModuleId::Settings,
    ];

    for m in modules {
        entries.push(PaletteEntry {
            title: format!("Go to {}", m.title()),
            subtitle: m
                .group()
                .map(|g| format!("{g:?} module"))
                .unwrap_or_else(|| "Core view".to_string()),
            action: PaletteAction::Navigate(m),
        });
    }

    for conv in &snapshot.conversations {
        entries.push(PaletteEntry {
            title: format!("Chat: {}", conv.title),
            subtitle: conv.last_message_preview.clone(),
            action: PaletteAction::Navigate(ModuleId::Chats),
        });
    }

    for note in &snapshot.notes {
        entries.push(PaletteEntry {
            title: format!("Note: {}", note.title),
            subtitle: format!("Tags: {}", note.tags.join(", ")),
            action: PaletteAction::Navigate(ModuleId::Notes),
        });
    }

    for task in &snapshot.todos {
        entries.push(PaletteEntry {
            title: format!("Todo: {}", task.title),
            subtitle: format!("List: {} ({:?})", task.list_name, task.priority),
            action: PaletteAction::Navigate(ModuleId::Todo),
        });
    }

    entries.push(PaletteEntry {
        title: "Theme: Switch to Dark".into(),
        subtitle: "Appearance".into(),
        action: PaletteAction::SetTheme(ThemeVariant::Dark),
    });
    entries.push(PaletteEntry {
        title: "Theme: Switch to Light".into(),
        subtitle: "Appearance".into(),
        action: PaletteAction::SetTheme(ThemeVariant::Light),
    });
    entries.push(PaletteEntry {
        title: "View: Toggle Sidebar".into(),
        subtitle: "Layout".into(),
        action: PaletteAction::ToggleSidebar,
    });
    entries.push(PaletteEntry {
        title: "View: Toggle Context Panel".into(),
        subtitle: "Layout".into(),
        action: PaletteAction::ToggleContextPanel,
    });

    if q.is_empty() {
        entries.truncate(12);
        return entries;
    }

    entries
        .into_iter()
        .filter(|e| {
            e.title.to_ascii_lowercase().contains(&q)
                || e.subtitle.to_ascii_lowercase().contains(&q)
        })
        .take(12)
        .collect()
}

//! Desktop UI shell crate for Keeftalk Desktop.
//!
//! Strictly isolated from database, cryptography, and network crates.
//! Full Iced GUI shell is implemented in Phase 2 (`feature/native-ui`).

use std::sync::Arc;

use keeftalk_core::KeeftalkCore;
use thiserror::Error;
use tracing::info;

#[derive(Debug, Error)]
pub enum UiError {
    #[error("core error: {0}")]
    Core(#[from] keeftalk_core::CoreError),
}

pub fn launch_shell(core: Arc<KeeftalkCore>, headless_check: bool) -> Result<(), UiError> {
    let cfg = core.config()?;
    let snap = core.snapshot()?;

    info!(
        mode = ?snap.mode,
        theme = ?cfg.ui.theme,
        chats = snap.conversations.len(),
        notes = snap.notes.len(),
        todos = snap.todos.len(),
        events = snap.calendar_events.len(),
        vault_items = snap.vault_items.len(),
        headless_check,
        "Keeftalk UI shell initialized"
    );

    if headless_check {
        println!(
            "Keeftalk Desktop ready (mode={:?}, chats={}, notes={}, todos={}, events={}, vault={})",
            snap.mode,
            snap.conversations.len(),
            snap.notes.len(),
            snap.todos.len(),
            snap.calendar_events.len(),
            snap.vault_items.len()
        );
    }

    Ok(())
}

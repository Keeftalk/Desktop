//! Keeftalk Desktop native Iced GUI shell.
//!
//! Strictly isolated from database, cryptographic, and network crates.
//! Communicates exclusively with `keeftalk-core` via typed snapshots and commands.

pub mod palette;
pub mod theme;

use std::sync::Arc;

use iced::alignment::{Horizontal, Vertical};
use iced::keyboard;
use iced::widget::{
    Space, button, checkbox, column, container, horizontal_rule, markdown, row, scrollable, stack,
    svg, text, text_input, vertical_rule,
};
use iced::{Alignment, Color, Element, Length, Subscription, Task};
use keeftalk_core::KeeftalkCore;
use keeftalk_core::model::{ModuleGroup, ModuleId, TodoPriority, WorkspaceSnapshot};
use keeftalk_settings::{AppMode, ThemeVariant};
use thiserror::Error;
use tracing::info;

use crate::palette::{PaletteAction, search_palette};
use crate::theme::{KeeftalkPalette, build_iced_theme, palette_for, parse_hex_color};

const KEEFTALK_LOGO_SVG: &[u8] = include_bytes!("../../../assets/icons/keeftalk_logo.svg");

#[derive(Debug, Error)]
pub enum UiError {
    #[error("core error: {0}")]
    Core(#[from] keeftalk_core::CoreError),

    #[error("window runtime error: {0}")]
    Runtime(String),
}

#[derive(Debug, Clone)]
pub enum Message {
    SelectModule(ModuleId),
    SelectChat(usize),
    SelectNote(usize),
    ToggleTodoCompleted(usize, bool),
    NewTodoTitleChanged(String),
    CreateTodo,
    ComposerChanged(String),
    SendMessage,
    ToggleSidebar,
    ToggleContextPanel,
    SelectTheme(ThemeVariant),
    ToggleCommandPalette,
    CloseCommandPalette,
    CommandQueryChanged(String),
    ExecutePaletteAction(PaletteAction),
    CommandPaletteSubmit,
    FontScaleChanged(f32),
}

pub struct KeeftalkApp {
    core: Arc<KeeftalkCore>,
    snapshot: WorkspaceSnapshot,
    active_module: ModuleId,
    selected_chat_idx: usize,
    selected_note_idx: usize,
    composer_text: String,
    new_todo_title: String,
    sidebar_collapsed: bool,
    context_panel_open: bool,
    theme_variant: ThemeVariant,
    custom_accent: Option<Color>,
    font_scale: f32,
    command_palette_open: bool,
    command_query: String,
    note_markdown_items: Vec<markdown::Item>,
}

impl KeeftalkApp {
    pub fn from_core(core: Arc<KeeftalkCore>) -> Result<Self, UiError> {
        let cfg = core.config()?;
        let snapshot = core.snapshot()?;
        let custom_accent = parse_hex_color(&cfg.ui.accent_color);
        let initial_md = snapshot
            .notes
            .first()
            .map(|n| markdown::parse(&n.markdown).collect())
            .unwrap_or_default();

        Ok(Self {
            core,
            snapshot,
            active_module: ModuleId::Home,
            selected_chat_idx: 0,
            selected_note_idx: 0,
            composer_text: String::new(),
            new_todo_title: String::new(),
            sidebar_collapsed: cfg.ui.sidebar_collapsed,
            context_panel_open: true,
            theme_variant: cfg.ui.theme,
            custom_accent,
            font_scale: cfg.ui.font_scale,
            command_palette_open: false,
            command_query: String::new(),
            note_markdown_items: initial_md,
        })
    }

    fn title(&self) -> String {
        match self.snapshot.mode {
            AppMode::Mock => format!("Keeftalk Desktop [Mock Mode] — {}", self.active_module.title()),
            AppMode::Offline => format!("Keeftalk Desktop [Offline] — {}", self.active_module.title()),
            AppMode::Cloud => format!("Keeftalk Desktop — {}", self.active_module.title()),
        }
    }

    fn theme(&self) -> iced::Theme {
        build_iced_theme(self.theme_variant, self.custom_accent)
    }

    fn palette(&self) -> KeeftalkPalette {
        palette_for(self.theme_variant, self.custom_accent)
    }

    fn subscription(&self) -> Subscription<Message> {
        keyboard::on_key_press(|key, modifiers| match key {
            keyboard::Key::Character(ref c) if c.eq_ignore_ascii_case("k") && modifiers.command() => {
                Some(Message::ToggleCommandPalette)
            }
            keyboard::Key::Named(keyboard::key::Named::Escape) => {
                Some(Message::CloseCommandPalette)
            }
            keyboard::Key::Character(ref c) if c == "," && modifiers.command() => {
                Some(Message::SelectModule(ModuleId::Settings))
            }
            _ => None,
        })
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SelectModule(module) => {
                self.active_module = module;
                self.command_palette_open = false;
            }
            Message::SelectChat(idx) => {
                if idx < self.snapshot.conversations.len() {
                    self.selected_chat_idx = idx;
                }
            }
            Message::SelectNote(idx) => {
                if let Some(note) = self.snapshot.notes.get(idx) {
                    self.selected_note_idx = idx;
                    self.note_markdown_items = markdown::parse(&note.markdown).collect();
                }
            }
            Message::ToggleTodoCompleted(idx, completed) => {
                if let Some(task) = self.snapshot.todos.get(idx)
                    && let Ok(next) = self.core.toggle_todo(&task.id, completed)
                {
                    self.snapshot = next;
                }
            }
            Message::NewTodoTitleChanged(val) => {
                self.new_todo_title = val;
            }
            Message::CreateTodo => {
                let title = self.new_todo_title.trim().to_string();
                if !title.is_empty()
                    && let Ok((_, next)) =
                        self.core.create_todo(&title, "General", TodoPriority::Medium, None)
                {
                    self.snapshot = next;
                    self.new_todo_title.clear();
                }
            }
            Message::ComposerChanged(val) => {
                self.composer_text = val;
            }
            Message::SendMessage => {
                let text_trimmed = self.composer_text.trim().to_string();
                if !text_trimmed.is_empty()
                    && let Some(conv) = self.snapshot.conversations.get(self.selected_chat_idx)
                    && let Ok(next) = self.core.send_chat_message(&conv.id, &text_trimmed)
                {
                    self.snapshot = next;
                    self.composer_text.clear();
                }
            }
            Message::ToggleSidebar => {
                self.sidebar_collapsed = !self.sidebar_collapsed;
            }
            Message::ToggleContextPanel => {
                self.context_panel_open = !self.context_panel_open;
            }
            Message::SelectTheme(variant) => {
                self.theme_variant = variant;
                self.custom_accent = None;
            }
            Message::ToggleCommandPalette => {
                self.command_palette_open = !self.command_palette_open;
                if self.command_palette_open {
                    self.command_query.clear();
                }
            }
            Message::CloseCommandPalette => {
                self.command_palette_open = false;
            }
            Message::CommandQueryChanged(q) => {
                self.command_query = q;
            }
            Message::ExecutePaletteAction(action) => {
                self.apply_palette_action(action);
            }
            Message::CommandPaletteSubmit => {
                if let Some(first) = search_palette(&self.command_query, &self.snapshot).into_iter().next() {
                    self.apply_palette_action(first.action);
                }
            }
            Message::FontScaleChanged(scale) => {
                self.font_scale = scale.clamp(0.85, 1.4);
            }
        }
        Task::none()
    }

    fn apply_palette_action(&mut self, action: PaletteAction) {
        match action {
            PaletteAction::Navigate(module) => self.active_module = module,
            PaletteAction::SetTheme(variant) => {
                self.theme_variant = variant;
                self.custom_accent = None;
            }
            PaletteAction::ToggleSidebar => self.sidebar_collapsed = !self.sidebar_collapsed,
            PaletteAction::ToggleContextPanel => self.context_panel_open = !self.context_panel_open,
        }
        self.command_palette_open = false;
    }

    pub fn view(&self) -> Element<'_, Message> {
        let p = self.palette();
        let top_bar = self.view_top_bar(p);
        let sidebar = self.view_sidebar(p);
        let main_content = self.view_main_content(p);

        let mut body_row = row![sidebar, vertical_rule(1), main_content].height(Length::Fill);
        if self.context_panel_open {
            body_row = body_row
                .push(vertical_rule(1))
                .push(self.view_context_panel(p));
        }

        let base_layout = container(column![top_bar, horizontal_rule(1), body_row])
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: Some(p.background.into()),
                text_color: Some(p.text_primary),
                ..Default::default()
            });

        if self.command_palette_open {
            stack![base_layout, self.view_command_palette(p)].into()
        } else {
            base_layout.into()
        }
    }

    fn view_top_bar(&self, p: KeeftalkPalette) -> Element<'_, Message> {
        let logo = svg(svg::Handle::from_memory(KEEFTALK_LOGO_SVG))
            .width(26)
            .height(26);

        let mode_label = match self.snapshot.mode {
            AppMode::Mock => "MOCK MODE",
            AppMode::Offline => "OFFLINE",
            AppMode::Cloud => "CLOUD",
        };

        let mode_color = match self.snapshot.mode {
            AppMode::Mock => p.warning,
            AppMode::Offline => p.success,
            AppMode::Cloud => p.primary,
        };

        let search_btn = button(text("Search or Command (Ctrl+K)").size(13).color(p.text_muted))
            .padding([6, 14])
            .on_press(Message::ToggleCommandPalette);

        let sidebar_btn = button(text(if self.sidebar_collapsed { "Expand Nav" } else { "Collapse Nav" }).size(12))
            .padding([6, 10])
            .on_press(Message::ToggleSidebar);

        let ctx_btn = button(text(if self.context_panel_open { "Hide Info" } else { "Show Info" }).size(12))
            .padding([6, 10])
            .on_press(Message::ToggleContextPanel);

        container(
            row![
                sidebar_btn,
                logo,
                text("Keeftalk").size(16),
                text(format!("/ {}", self.active_module.title()))
                    .size(14)
                    .color(p.text_muted),
                Space::with_width(Length::Fill),
                search_btn,
                Space::with_width(12),
                container(text(mode_label).size(11).color(Color::WHITE))
                    .padding([4, 8])
                    .style(move |_| container::Style {
                        background: Some(mode_color.into()),
                        border: iced::Border {
                            radius: 6.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ctx_btn,
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        )
        .padding([10, 14])
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(p.surface.into()),
            ..Default::default()
        })
        .into()
    }

    fn view_sidebar(&self, p: KeeftalkPalette) -> Element<'_, Message> {
        let width = if self.sidebar_collapsed { 72.0 } else { 220.0 };

        let groups: &[(&str, &[ModuleId])] = &[
            ("Core", &[ModuleId::Home]),
            ("Communication", &[ModuleId::Chats, ModuleId::Calls, ModuleId::SmsInfo]),
            (
                "Productivity",
                &[
                    ModuleId::Notes,
                    ModuleId::Calendar,
                    ModuleId::Todo,
                    ModuleId::Habits,
                    ModuleId::Books,
                    ModuleId::Docs,
                ],
            ),
            (
                "Security",
                &[ModuleId::Vault, ModuleId::Secrets, ModuleId::SecurityCenter],
            ),
            ("Finance", &[ModuleId::Wallet, ModuleId::Finances, ModuleId::Market]),
            ("Discover", &[ModuleId::Feed, ModuleId::Browser]),
            ("Personal", &[ModuleId::Email, ModuleId::Maps, ModuleId::Din]),
        ];

        let mut items = column![].spacing(6).padding(10);

        for (group_name, modules) in groups {
            if !self.sidebar_collapsed {
                items = items.push(
                    text(*group_name)
                        .size(11)
                        .color(p.text_muted),
                );
            }
            for &module in *modules {
                let is_active = self.active_module == module;
                let label = if self.sidebar_collapsed {
                    module.title().chars().take(2).collect::<String>()
                } else {
                    module.title().to_string()
                };

                let btn = button(text(label).size(13))
                    .width(Length::Fill)
                    .padding([7, 10])
                    .style(move |_theme, _status| button::Style {
                        background: Some(if is_active { p.primary } else { Color::TRANSPARENT }.into()),
                        text_color: if is_active { Color::WHITE } else { p.text_primary },
                        border: iced::Border {
                            radius: 6.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .on_press(Message::SelectModule(module));
                items = items.push(btn);
            }
            items = items.push(Space::with_height(4));
        }

        let settings_active = self.active_module == ModuleId::Settings;
        items = items.push(horizontal_rule(1)).push(
            button(text(if self.sidebar_collapsed { "St" } else { "Settings" }).size(13))
                .width(Length::Fill)
                .padding([8, 10])
                .style(move |_theme, _status| button::Style {
                    background: Some(if settings_active { p.primary } else { Color::TRANSPARENT }.into()),
                    text_color: if settings_active { Color::WHITE } else { p.text_primary },
                    border: iced::Border {
                        radius: 6.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .on_press(Message::SelectModule(ModuleId::Settings)),
        );

        container(scrollable(items))
            .width(width)
            .height(Length::Fill)
            .style(move |_| container::Style {
                background: Some(p.surface.into()),
                ..Default::default()
            })
            .into()
    }

    fn view_main_content(&self, p: KeeftalkPalette) -> Element<'_, Message> {
        let content: Element<'_, Message> = match self.active_module {
            ModuleId::Home => self.view_home(p),
            ModuleId::Chats => self.view_chats(p),
            ModuleId::Notes => self.view_notes(p),
            ModuleId::Todo => self.view_todo(p),
            ModuleId::Calendar => self.view_calendar(p),
            ModuleId::Vault => self.view_vault(p),
            ModuleId::Settings => self.view_settings(p),
            ModuleId::SmsInfo => self.view_sms_limitation(p),
            other => self.view_planned_module(other, p),
        };

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .into()
    }

    fn view_home(&self, p: KeeftalkPalette) -> Element<'_, Message> {
        let stats = row![
            self.stat_card("Conversations", self.snapshot.conversations.len().to_string(), ModuleId::Chats, p),
            self.stat_card("Encrypted Notes", self.snapshot.notes.len().to_string(), ModuleId::Notes, p),
            self.stat_card("Tasks", self.snapshot.todos.len().to_string(), ModuleId::Todo, p),
            self.stat_card("Vault Items", self.snapshot.vault_items.len().to_string(), ModuleId::Vault, p),
        ]
        .spacing(12);

        let quick_actions = row![
            button(text("Open Chats").size(13)).padding([8, 14]).on_press(Message::SelectModule(ModuleId::Chats)),
            button(text("Open Notes").size(13)).padding([8, 14]).on_press(Message::SelectModule(ModuleId::Notes)),
            button(text("Open Todo").size(13)).padding([8, 14]).on_press(Message::SelectModule(ModuleId::Todo)),
            button(text("Command Palette (Ctrl+K)").size(13)).padding([8, 14]).on_press(Message::ToggleCommandPalette),
        ]
        .spacing(10);

        column![
            text("Welcome to Keeftalk Desktop").size(24),
            text("Privacy-first, offline-first Super App • Zero ads, zero tracking.")
                .size(14)
                .color(p.text_muted),
            Space::with_height(8),
            stats,
            Space::with_height(12),
            text("Quick Actions").size(16),
            quick_actions,
        ]
        .spacing(12)
        .into()
    }

    fn stat_card<'a>(
        &self,
        label: &'a str,
        value: String,
        target: ModuleId,
        p: KeeftalkPalette,
    ) -> Element<'a, Message> {
        button(
            column![
                text(value).size(24).color(p.primary),
                text(label).size(13).color(p.text_muted),
            ]
            .spacing(4),
        )
        .padding(14)
        .width(Length::Fill)
        .style(move |_theme, _status| button::Style {
            background: Some(p.surface.into()),
            text_color: p.text_primary,
            border: iced::Border {
                color: p.border,
                width: 1.0,
                radius: 10.0.into(),
            },
            ..Default::default()
        })
        .on_press(Message::SelectModule(target))
        .into()
    }

    fn view_chats(&self, p: KeeftalkPalette) -> Element<'_, Message> {
        let mut list = column![text("Conversations").size(16)].spacing(8);
        for (idx, conv) in self.snapshot.conversations.iter().enumerate() {
            let selected = idx == self.selected_chat_idx;
            list = list.push(
                button(
                    column![
                        text(&conv.title).size(14),
                        text(&conv.last_message_preview).size(12).color(p.text_muted),
                    ]
                    .spacing(3),
                )
                .width(Length::Fill)
                .padding(10)
                .style(move |_theme, _status| button::Style {
                    background: Some(if selected { p.surface_alt } else { p.surface }.into()),
                    text_color: p.text_primary,
                    border: iced::Border {
                        color: if selected { p.primary } else { p.border },
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                })
                .on_press(Message::SelectChat(idx)),
            );
        }

        let thread: Element<'_, Message> = if let Some(conv) = self.snapshot.conversations.get(self.selected_chat_idx) {
            let mut msgs = column![].spacing(8);
            for m in &conv.messages {
                let bubble_bg = if m.is_outgoing { p.primary } else { p.surface_alt };
                let bubble_fg = if m.is_outgoing { Color::WHITE } else { p.text_primary };
                msgs = msgs.push(
                    container(
                        column![
                            text(format!("{} • {}", m.sender_name, if m.encrypted { "E2EE" } else { "Plain" }))
                                .size(11)
                                .color(if m.is_outgoing { Color::from_rgba(1.0, 1.0, 1.0, 0.8) } else { p.text_muted }),
                            text(&m.body).size(14).color(bubble_fg),
                        ]
                        .spacing(3),
                    )
                    .padding(10)
                    .width(Length::FillPortion(3))
                    .style(move |_| container::Style {
                        background: Some(bubble_bg.into()),
                        border: iced::Border {
                            radius: 8.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                );
            }

            let composer = row![
                text_input("Write an encrypted message...", &self.composer_text)
                    .on_input(Message::ComposerChanged)
                    .on_submit(Message::SendMessage)
                    .padding(10)
                    .width(Length::Fill),
                button(text("Send").size(13))
                    .padding([10, 16])
                    .on_press(Message::SendMessage),
            ]
            .spacing(8);

            column![
                text(&conv.title).size(18),
                horizontal_rule(1),
                scrollable(msgs).height(Length::Fill),
                composer,
            ]
            .spacing(10)
            .into()
        } else {
            text("No conversations yet.").color(p.text_muted).into()
        };

        row![
            container(scrollable(list)).width(260).height(Length::Fill),
            vertical_rule(1),
            container(thread).width(Length::Fill).height(Length::Fill).padding([0, 14]),
        ]
        .spacing(12)
        .into()
    }

    fn view_notes(&self, p: KeeftalkPalette) -> Element<'_, Message> {
        let mut list = column![text("My Notes").size(16)].spacing(8);
        for (idx, note) in self.snapshot.notes.iter().enumerate() {
            let selected = idx == self.selected_note_idx;
            list = list.push(
                button(
                    column![
                        text(&note.title).size(14),
                        text(note.tags.join(", ")).size(12).color(p.text_muted),
                    ]
                    .spacing(3),
                )
                .width(Length::Fill)
                .padding(10)
                .style(move |_theme, _status| button::Style {
                    background: Some(if selected { p.surface_alt } else { p.surface }.into()),
                    text_color: p.text_primary,
                    border: iced::Border {
                        color: if selected { p.primary } else { p.border },
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                })
                .on_press(Message::SelectNote(idx)),
            );
        }

        let preview: Element<'_, Message> = if let Some(note) = self.snapshot.notes.get(self.selected_note_idx) {
            let md_view = markdown::view(
                &self.note_markdown_items,
                markdown::Settings::default(),
                markdown::Style::from_palette(self.theme().palette()),
            )
            .map(|_url| Message::ToggleContextPanel);

            column![
                text(&note.title).size(20),
                text(format!("Tags: {}", note.tags.join(", "))).size(12).color(p.text_muted),
                horizontal_rule(1),
                scrollable(md_view).height(Length::Fill),
            ]
            .spacing(10)
            .into()
        } else {
            text("Select a note to preview.").color(p.text_muted).into()
        };

        row![
            container(scrollable(list)).width(260).height(Length::Fill),
            vertical_rule(1),
            container(preview).width(Length::Fill).height(Length::Fill).padding([0, 14]),
        ]
        .spacing(12)
        .into()
    }

    fn view_todo(&self, p: KeeftalkPalette) -> Element<'_, Message> {
        let add_bar = row![
            text_input("Add a new task...", &self.new_todo_title)
                .on_input(Message::NewTodoTitleChanged)
                .on_submit(Message::CreateTodo)
                .padding(10)
                .width(Length::Fill),
            button(text("Add Task").size(13))
                .padding([10, 16])
                .on_press(Message::CreateTodo),
        ]
        .spacing(8);

        let mut list = column![text("My Todo").size(20), add_bar].spacing(10);
        for (idx, task) in self.snapshot.todos.iter().enumerate() {
            let priority_label = match task.priority {
                TodoPriority::Low => "Low",
                TodoPriority::Medium => "Medium",
                TodoPriority::High => "High",
                TodoPriority::Urgent => "Urgent",
            };
            let check = checkbox(&task.title, task.completed)
                .on_toggle(move |checked| Message::ToggleTodoCompleted(idx, checked));

            list = list.push(
                container(
                    row![
                        check,
                        Space::with_width(Length::Fill),
                        text(format!("{} • {}", task.list_name, priority_label))
                            .size(12)
                            .color(p.text_muted),
                    ]
                    .align_y(Alignment::Center),
                )
                .padding(10)
                .width(Length::Fill)
                .style(move |_| container::Style {
                    background: Some(p.surface.into()),
                    border: iced::Border {
                        color: p.border,
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                }),
            );
        }
        scrollable(list).into()
    }

    fn view_calendar(&self, p: KeeftalkPalette) -> Element<'_, Message> {
        let mut items = column![text("My Calendar").size(20)].spacing(10);
        for ev in &self.snapshot.calendar_events {
            items = items.push(
                container(
                    column![
                        text(&ev.title).size(15),
                        text(&ev.description).size(13).color(p.text_muted),
                        text(format!(
                            "{} -> {}",
                            ev.start_time.format("%Y-%m-%d %H:%M UTC"),
                            ev.end_time.format("%H:%M UTC")
                        ))
                        .size(12)
                        .color(p.primary),
                    ]
                    .spacing(4),
                )
                .padding(12)
                .width(Length::Fill)
                .style(move |_| container::Style {
                    background: Some(p.surface.into()),
                    border: iced::Border {
                        color: p.border,
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                }),
            );
        }
        scrollable(items).into()
    }

    fn view_vault(&self, p: KeeftalkPalette) -> Element<'_, Message> {
        let mut items = column![
            text("My Vault").size(20),
            text("Zero-knowledge encrypted files (AES-256-GCM + .ktb backups)")
                .size(13)
                .color(p.text_muted),
        ]
        .spacing(10);

        for item in &self.snapshot.vault_items {
            items = items.push(
                container(
                    row![
                        column![
                            text(&item.name).size(14),
                            text(format!("Folder: {} • {} bytes", item.folder, item.size_bytes))
                                .size(12)
                                .color(p.text_muted),
                        ]
                        .spacing(3),
                        Space::with_width(Length::Fill),
                        text(if item.encrypted { "Encrypted" } else { "Plain" })
                            .size(12)
                            .color(p.success),
                    ]
                    .align_y(Alignment::Center),
                )
                .padding(12)
                .width(Length::Fill)
                .style(move |_| container::Style {
                    background: Some(p.surface.into()),
                    border: iced::Border {
                        color: p.border,
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                }),
            );
        }
        scrollable(items).into()
    }

    fn view_settings(&self, p: KeeftalkPalette) -> Element<'_, Message> {
        let themes = [
            ("Dark", ThemeVariant::Dark),
            ("Light", ThemeVariant::Light),
            ("Day", ThemeVariant::Day),
            ("Pinky", ThemeVariant::Pinky),
            ("Masculine", ThemeVariant::Masculine),
        ];

        let mut theme_row = row![].spacing(8);
        for (label, variant) in themes {
            let active = self.theme_variant == variant;
            theme_row = theme_row.push(
                button(text(label).size(13))
                    .padding([8, 12])
                    .style(move |_theme, _status| button::Style {
                        background: Some(if active { p.primary } else { p.surface }.into()),
                        text_color: if active { Color::WHITE } else { p.text_primary },
                        border: iced::Border {
                            color: p.border,
                            width: 1.0,
                            radius: 6.0.into(),
                        },
                        ..Default::default()
                    })
                    .on_press(Message::SelectTheme(variant)),
            );
        }

        column![
            text("Settings").size(22),
            horizontal_rule(1),
            text("Appearance & Theme").size(16),
            theme_row,
            Space::with_height(8),
            text("Shortcuts").size(16),
            text("Ctrl/Cmd + K: Global Search & Command Palette").size(13).color(p.text_muted),
            text("Ctrl/Cmd + ,: Open Settings").size(13).color(p.text_muted),
            text("Escape: Close Command Palette").size(13).color(p.text_muted),
        ]
        .spacing(10)
        .into()
    }

    fn view_sms_limitation(&self, p: KeeftalkPalette) -> Element<'_, Message> {
        container(
            column![
                text("SMS / MMS — Desktop Platform Limitation").size(20),
                text("Desktop operating systems (Windows, Linux, macOS) do not expose native carrier SIM telephony APIs.")
                    .size(14)
                    .color(p.text_muted),
                text("Keeftalk Desktop never simulates or fakes carrier SMS sending. Synced Android SMS history will appear read-only when connected.")
                    .size(13)
                    .color(p.warning),
            ]
            .spacing(10),
        )
        .padding(16)
        .style(move |_| container::Style {
            background: Some(p.surface.into()),
            border: iced::Border {
                color: p.border,
                width: 1.0,
                radius: 10.0.into(),
            },
            ..Default::default()
        })
        .into()
    }

    fn view_planned_module(&self, module: ModuleId, p: KeeftalkPalette) -> Element<'_, Message> {
        let group_label = module
            .group()
            .unwrap_or(ModuleGroup::Personal);
        container(
            column![
                text(module.title()).size(22),
                text(format!("Module Group: {group_label:?}")).size(13).color(p.primary),
                text("Scheduled in Phase 7 of ROADMAP.md. Local-first implementation will replace this status card once its backend/local repository is wired.")
                    .size(14)
                    .color(p.text_muted),
            ]
            .spacing(8),
        )
        .padding(16)
        .style(move |_| container::Style {
            background: Some(p.surface.into()),
            border: iced::Border {
                color: p.border,
                width: 1.0,
                radius: 10.0.into(),
            },
            ..Default::default()
        })
        .into()
    }

    fn view_context_panel(&self, p: KeeftalkPalette) -> Element<'_, Message> {
        column![
            text("Context & Security").size(15),
            horizontal_rule(1),
            text(format!("Mode: {:?}", self.snapshot.mode)).size(13),
            text(format!("Sync: {:?}", self.snapshot.sync.status)).size(13).color(p.text_muted),
            text(format!("Pending Queue: {}", self.snapshot.sync.pending_items)).size(13).color(p.text_muted),
            Space::with_height(8),
            text("Encryption").size(14),
            text("AES-256-GCM + Argon2id (64 MiB)").size(12).color(p.success),
        ]
        .spacing(8)
        .padding(14)
        .width(240)
        .height(Length::Fill)
        .into()
    }

    fn view_command_palette(&self, p: KeeftalkPalette) -> Element<'_, Message> {
        let results = search_palette(&self.command_query, &self.snapshot);
        let mut rows = column![].spacing(6);

        for entry in results {
            rows = rows.push(
                button(
                    row![
                        text(entry.title).size(13),
                        Space::with_width(Length::Fill),
                        text(entry.subtitle).size(11).color(p.text_muted),
                    ]
                    .align_y(Alignment::Center),
                )
                .width(Length::Fill)
                .padding([8, 10])
                .on_press(Message::ExecutePaletteAction(entry.action)),
            );
        }

        let modal = container(
            column![
                text_input("Type a command, module, chat, or note...", &self.command_query)
                    .on_input(Message::CommandQueryChanged)
                    .on_submit(Message::CommandPaletteSubmit)
                    .padding(10),
                horizontal_rule(1),
                scrollable(rows).height(280),
            ]
            .spacing(10),
        )
        .width(560)
        .padding(14)
        .style(move |_| container::Style {
            background: Some(p.surface.into()),
            border: iced::Border {
                color: p.primary,
                width: 1.5,
                radius: 12.0.into(),
            },
            ..Default::default()
        });

        container(modal)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Top)
            .padding(iced::Padding {
                top: 72.0,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            })
            .style(|_| container::Style {
                background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.55).into()),
                ..Default::default()
            })
            .into()
    }
}

pub fn launch_shell(core: Arc<KeeftalkCore>, headless_check: bool) -> Result<(), UiError> {
    let cfg = core.config()?;
    let snap = core.snapshot()?;

    info!(
        mode = ?snap.mode,
        theme = ?cfg.ui.theme,
        headless_check,
        "Launching Keeftalk Iced desktop shell"
    );

    if headless_check {
        println!(
            "Keeftalk Desktop UI ready (mode={:?}, theme={:?}, chats={}, notes={}, todos={}, events={}, vault={})",
            snap.mode,
            cfg.ui.theme,
            snap.conversations.len(),
            snap.notes.len(),
            snap.todos.len(),
            snap.calendar_events.len(),
            snap.vault_items.len()
        );
        return Ok(());
    }

    let initial_core = Arc::clone(&core);
    iced::application(KeeftalkApp::title, KeeftalkApp::update, KeeftalkApp::view)
        .theme(KeeftalkApp::theme)
        .subscription(KeeftalkApp::subscription)
        .window_size((1280.0, 820.0))
        .run_with(move || {
            let app = KeeftalkApp::from_core(Arc::clone(&initial_core))
                .expect("core state validated before UI launch");
            (app, Task::none())
        })
        .map_err(|e| UiError::Runtime(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use keeftalk_settings::AppConfig;

    fn test_app() -> KeeftalkApp {
        let cfg = AppConfig::load(None).expect("config");
        let core = Arc::new(KeeftalkCore::bootstrap_mock(cfg).expect("bootstrap"));
        KeeftalkApp::from_core(core).expect("ui init")
    }

    #[test]
    fn navigation_and_theme_updates_work() {
        let mut app = test_app();
        assert_eq!(app.active_module, ModuleId::Home);

        let _ = app.update(Message::SelectModule(ModuleId::Notes));
        assert_eq!(app.active_module, ModuleId::Notes);

        let _ = app.update(Message::SelectTheme(ThemeVariant::Pinky));
        assert_eq!(app.theme_variant, ThemeVariant::Pinky);
    }

    #[test]
    fn command_palette_filters_and_executes() {
        let mut app = test_app();
        let _ = app.update(Message::ToggleCommandPalette);
        assert!(app.command_palette_open);

        let _ = app.update(Message::CommandQueryChanged("vault".into()));
        let _ = app.update(Message::CommandPaletteSubmit);
        assert!(!app.command_palette_open);
        assert_eq!(app.active_module, ModuleId::Vault);
    }
}

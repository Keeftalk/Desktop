//! Embedded SVG icons for Keeftalk Desktop UI navigation and module branding.
//! Guarantees crisp, system-independent icon rendering without relying on OS emoji font fallbacks.

use iced::widget::svg::{self, Svg};
use keeftalk_core::model::ModuleId;

pub const HOME_SVG: &[u8] = b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='#6c63ff' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'><path d='M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z'/><polyline points='9 22 9 12 15 12 15 22'/></svg>";
pub const CHATS_SVG: &[u8] = b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='#6c63ff' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'><path d='M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z'/></svg>";
pub const CALLS_SVG: &[u8] = b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='#2ecc71' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'><path d='M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72 12.84 12.84 0 0 0 .7 2.81 2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45 12.84 12.84 0 0 0 2.81.7A2 2 0 0 1 22 16.92z'/></svg>";
pub const SMS_SVG: &[u8] = b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='#ffb067' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'><rect x='5' y='2' width='14' height='20' rx='2' ry='2'/><line x1='12' y1='18' x2='12.01' y2='18'/></svg>";
pub const NOTES_SVG: &[u8] = b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='#4a90d9' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'><path d='M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z'/><polyline points='14 2 14 8 20 8'/><line x1='16' y1='13' x2='8' y2='13'/><line x1='16' y1='17' x2='8' y2='17'/><polyline points='10 9 9 9 8 9'/></svg>";
pub const CALENDAR_SVG: &[u8] = b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='#2ecc71' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'><rect x='3' y='4' width='18' height='18' rx='2' ry='2'/><line x1='16' y1='2' x2='16' y2='6'/><line x1='8' y1='2' x2='8' y2='6'/><line x1='3' y1='10' x2='21' y2='10'/></svg>";
pub const TODO_SVG: &[u8] = b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='#2ecc71' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'><polyline points='9 11 12 14 22 4'/><path d='M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11'/></svg>";
pub const HABITS_SVG: &[u8] = b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='#ffb067' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'><polyline points='23 4 23 10 17 10'/><path d='M20.49 15a9 9 0 1 1-2.12-9.36L23 10'/></svg>";
pub const BOOKS_SVG: &[u8] = b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='#4a90d9' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'><path d='M4 19.5A2.5 2.5 0 0 1 6.5 17H20'/><path d='M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z'/></svg>";
pub const DOCS_SVG: &[u8] = b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='#6c63ff' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'><path d='M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z'/><polyline points='13 2 13 9 20 9'/></svg>";
pub const VAULT_SVG: &[u8] = b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='#ffb067' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'><rect x='3' y='11' width='18' height='11' rx='2' ry='2'/><path d='M7 11V7a5 5 0 0 1 10 0v4'/></svg>";
pub const SECRETS_SVG: &[u8] = b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='#ff6b6b' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'><path d='M21 2l-2 2m-7.61 7.61a5.5 5.5 0 1 1-7.778 7.778 5.5 5.5 0 0 1 7.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4'/></svg>";
pub const SECURITY_SVG: &[u8] = b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='#2ecc71' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'><path d='M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z'/></svg>";
pub const WALLET_SVG: &[u8] = b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='#2ecc71' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'><rect x='1' y='4' width='22' height='16' rx='2' ry='2'/><line x1='1' y1='10' x2='23' y2='10'/></svg>";
pub const FINANCES_SVG: &[u8] = b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='#4a90d9' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'><line x1='18' y1='20' x2='18' y2='10'/><line x1='12' y1='20' x2='12' y2='4'/><line x1='6' y1='20' x2='6' y2='14'/></svg>";
pub const MARKET_SVG: &[u8] = b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='#ff4081' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'><path d='M6 2L3 6v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V6l-3-4z'/><line x1='3' y1='6' x2='21' y2='6'/><path d='M16 10a4 4 0 0 1-8 0'/></svg>";
pub const FEED_SVG: &[u8] = b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='#ffb067' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'><path d='M4 11a9 9 0 0 1 9 9'/><path d='M4 4a16 16 0 0 1 16 16'/><circle cx='5' cy='19' r='1'/></svg>";
pub const BROWSER_SVG: &[u8] = b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='#4a90d9' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'><circle cx='12' cy='12' r='10'/><line x1='2' y1='12' x2='22' y2='12'/><path d='M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z'/></svg>";
pub const EMAIL_SVG: &[u8] = b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='#6c63ff' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'><path d='M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z'/><polyline points='22,6 12,13 2,6'/></svg>";
pub const MAPS_SVG: &[u8] = b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='#2ecc71' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'><path d='M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0 1 18 0z'/><circle cx='12' cy='10' r='3'/></svg>";
pub const DIN_SVG: &[u8] = b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='#2ecc71' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'><path d='M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z'/></svg>";
pub const SETTINGS_SVG: &[u8] = b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='#8e92a8' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'><circle cx='12' cy='12' r='3'/><path d='M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z'/></svg>";

pub fn module_icon_bytes(module: ModuleId) -> &'static [u8] {
    match module {
        ModuleId::Home => HOME_SVG,
        ModuleId::Chats => CHATS_SVG,
        ModuleId::Calls => CALLS_SVG,
        ModuleId::SmsInfo => SMS_SVG,
        ModuleId::Notes => NOTES_SVG,
        ModuleId::Calendar => CALENDAR_SVG,
        ModuleId::Todo => TODO_SVG,
        ModuleId::Habits => HABITS_SVG,
        ModuleId::Books => BOOKS_SVG,
        ModuleId::Docs => DOCS_SVG,
        ModuleId::Vault => VAULT_SVG,
        ModuleId::Secrets => SECRETS_SVG,
        ModuleId::SecurityCenter => SECURITY_SVG,
        ModuleId::Wallet => WALLET_SVG,
        ModuleId::Finances => FINANCES_SVG,
        ModuleId::Market => MARKET_SVG,
        ModuleId::Feed => FEED_SVG,
        ModuleId::Browser => BROWSER_SVG,
        ModuleId::Email => EMAIL_SVG,
        ModuleId::Maps => MAPS_SVG,
        ModuleId::Din => DIN_SVG,
        ModuleId::Settings => SETTINGS_SVG,
    }
}

pub fn render_module_icon(module: ModuleId, size: u16) -> Svg<'static> {
    iced::widget::svg(svg::Handle::from_memory(module_icon_bytes(module)))
        .width(size)
        .height(size)
}

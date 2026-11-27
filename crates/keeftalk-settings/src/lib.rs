//! Layered configuration loader for Keeftalk Desktop.
//!
//! Configuration precedence (lowest to highest):
//! 1. Built-in `config/defaults.toml`
//! 2. Explicit config file (if present)
//! 3. Environment variables (`KEEFTALK_MODE`, `KEEFTALK_SUPABASE_URL`, etc.)

use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

const DEFAULT_CONFIG_TOML: &str = include_str!("../../../config/defaults.toml");

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("failed to read configuration file '{path}': {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse TOML configuration: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("invalid configuration: {0}")]
    Validation(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AppMode {
    /// Local demo mode with deterministic sample data and no network calls.
    Mock,
    /// Full local encrypted storage without requiring a cloud account.
    #[default]
    Offline,
    /// Connected to a configured Supabase + Cloudflare R2 backend.
    Cloud,
}

impl AppMode {
    pub fn from_env_str(raw: &str) -> Result<Self, SettingsError> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "mock" | "demo" => Ok(Self::Mock),
            "offline" | "local" => Ok(Self::Offline),
            "cloud" | "online" => Ok(Self::Cloud),
            other => Err(SettingsError::Validation(format!(
                "unsupported KEEFTALK_MODE '{other}'; expected mock, offline, or cloud"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemeVariant {
    Light,
    #[default]
    Dark,
    Day,
    Pinky,
    Masculine,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSection {
    pub mode: AppMode,
    pub language: String,
    pub check_updates: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendSection {
    pub supabase_url: String,
    pub supabase_anon_key: String,
    pub request_timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSection {
    pub theme: ThemeVariant,
    pub accent_color: String,
    pub sidebar_collapsed: bool,
    pub use_keef_atmosphere: bool,
    pub font_scale: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySection {
    pub auto_lock_seconds: u64,
    pub clipboard_clear_seconds: u64,
    pub require_os_keyring: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingSection {
    pub level: String,
    pub log_to_file: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub app: AppSection,
    pub backend: BackendSection,
    pub ui: UiSection,
    pub security: SecuritySection,
    pub logging: LoggingSection,
}

impl AppConfig {
    /// Loads default configuration, optionally overlays a TOML file, then applies environment overrides.
    pub fn load(config_path: Option<&Path>) -> Result<Self, SettingsError> {
        let mut cfg: Self = toml::from_str(DEFAULT_CONFIG_TOML)?;

        if let Some(path) = config_path
            && path.exists()
        {
            let raw = std::fs::read_to_string(path).map_err(|source| SettingsError::Io {
                path: path.display().to_string(),
                source,
            })?;
            let overlay: Self = toml::from_str(&raw)?;
            cfg = overlay;
        }

        cfg.apply_env_overrides()?;
        cfg.validate()?;
        Ok(cfg)
    }

    fn apply_env_overrides(&mut self) -> Result<(), SettingsError> {
        if let Ok(mode) = std::env::var("KEEFTALK_MODE")
            && !mode.trim().is_empty()
        {
            self.app.mode = AppMode::from_env_str(&mode)?;
        }
        if let Ok(url) = std::env::var("KEEFTALK_SUPABASE_URL")
            && !url.trim().is_empty()
        {
            self.backend.supabase_url = url.trim().to_string();
        }
        if let Ok(key) = std::env::var("KEEFTALK_SUPABASE_ANON_KEY")
            && !key.trim().is_empty()
        {
            self.backend.supabase_anon_key = key.trim().to_string();
        }
        if let Ok(level) = std::env::var("KEEFTALK_LOG_LEVEL")
            && !level.trim().is_empty()
        {
            self.logging.level = level.trim().to_string();
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<(), SettingsError> {
        if !(0.75..=2.0).contains(&self.ui.font_scale) {
            return Err(SettingsError::Validation(format!(
                "ui.font_scale ({}) must be between 0.75 and 2.0",
                self.ui.font_scale
            )));
        }

        if self.app.mode == AppMode::Cloud {
            if self.backend.supabase_url.trim().is_empty() {
                return Err(SettingsError::Validation(
                    "backend.supabase_url is required when app.mode is 'cloud'".into(),
                ));
            }
            if self.backend.supabase_anon_key.trim().is_empty() {
                return Err(SettingsError::Validation(
                    "backend.supabase_anon_key is required when app.mode is 'cloud'".into(),
                ));
            }
            Url::parse(&self.backend.supabase_url).map_err(|e| {
                SettingsError::Validation(format!("invalid backend.supabase_url: {e}"))
            })?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_defaults_without_network_or_secrets() {
        let cfg = AppConfig::load(None).expect("defaults should load cleanly");
        assert_eq!(cfg.app.mode, AppMode::Offline);
        assert_eq!(cfg.ui.theme, ThemeVariant::Dark);
        assert!(cfg.security.require_os_keyring);
    }

    #[test]
    fn cloud_mode_requires_backend_url_and_key() {
        let mut cfg = AppConfig::load(None).expect("defaults should load");
        cfg.app.mode = AppMode::Cloud;
        let err = cfg.validate().expect_err("cloud mode without URL must fail");
        assert!(err.to_string().contains("supabase_url"));
    }
}

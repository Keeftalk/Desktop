//! Keeftalk Desktop theme palettes derived from `Keeftalk/Android` design tokens.

use iced::Color;
use iced::theme::Palette;
use keeftalk_settings::ThemeVariant;

#[derive(Debug, Clone, Copy)]
pub struct KeeftalkPalette {
    pub background: Color,
    pub surface: Color,
    pub surface_alt: Color,
    pub sidebar: Color,
    pub border: Color,
    pub text_primary: Color,
    pub text_muted: Color,
    pub primary: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
}

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}

pub fn palette_for(variant: ThemeVariant, custom_accent: Option<Color>) -> KeeftalkPalette {
    let mut base = match variant {
        ThemeVariant::Dark => KeeftalkPalette {
            background: rgb(0x09, 0x0B, 0x10),
            surface: rgb(0x12, 0x15, 0x1E),
            surface_alt: rgb(0x1A, 0x1E, 0x2B),
            sidebar: rgb(0x0D, 0x10, 0x18),
            border: rgb(0x26, 0x2B, 0x3A),
            text_primary: rgb(0xEC, 0xEC, 0xF0),
            text_muted: rgb(0x8E, 0x92, 0xA8),
            primary: rgb(0x6C, 0x63, 0xFF),
            success: rgb(0x2E, 0xCC, 0x71),
            warning: rgb(0xFF, 0xB0, 0x67),
            danger: rgb(0xFF, 0x6B, 0x6B),
        },
        ThemeVariant::Light => KeeftalkPalette {
            background: rgb(0xF2, 0xF4, 0xF8),
            surface: rgb(0xFF, 0xFF, 0xFF),
            surface_alt: rgb(0xF8, 0xF9, 0xFC),
            sidebar: rgb(0x1A, 0x1A, 0x2E),
            border: rgb(0xE2, 0xE5, 0xEE),
            text_primary: rgb(0x11, 0x18, 0x27),
            text_muted: rgb(0x6B, 0x72, 0x80),
            primary: rgb(0x6C, 0x63, 0xFF),
            success: rgb(0x16, 0x8A, 0x4A),
            warning: rgb(0xD9, 0x77, 0x06),
            danger: rgb(0xD1, 0x2E, 0x2E),
        },
        ThemeVariant::Day => KeeftalkPalette {
            background: rgb(0xF0, 0xF2, 0xF5),
            surface: rgb(0xFF, 0xFF, 0xFF),
            surface_alt: rgb(0xE8, 0xEC, 0xF1),
            sidebar: rgb(0x0E, 0x2A, 0x3F),
            border: rgb(0xD6, 0xE6, 0xF7),
            text_primary: rgb(0x0E, 0x2A, 0x3F),
            text_muted: rgb(0x47, 0x65, 0x7B),
            primary: rgb(0x4A, 0x90, 0xD9),
            success: rgb(0x16, 0x8A, 0x4A),
            warning: rgb(0x8A, 0x4B, 0x00),
            danger: rgb(0xD1, 0x2E, 0x2E),
        },
        ThemeVariant::Pinky => KeeftalkPalette {
            background: rgb(0xFF, 0xF0, 0xF5),
            surface: rgb(0xFF, 0xFF, 0xFF),
            surface_alt: rgb(0xFF, 0xF5, 0xF8),
            sidebar: rgb(0x4A, 0x2A, 0x3A),
            border: rgb(0xF8, 0xD7, 0xE0),
            text_primary: rgb(0x4A, 0x2A, 0x3A),
            text_muted: rgb(0x7A, 0x4A, 0x5A),
            primary: rgb(0xFF, 0x40, 0x81),
            success: rgb(0x16, 0x8A, 0x4A),
            warning: rgb(0xD9, 0x77, 0x06),
            danger: rgb(0xE1, 0x1D, 0x48),
        },
        ThemeVariant::Masculine => KeeftalkPalette {
            background: rgb(0x0A, 0x0E, 0x17),
            surface: rgb(0x13, 0x1C, 0x2E),
            surface_alt: rgb(0x1A, 0x27, 0x44),
            sidebar: rgb(0x07, 0x0B, 0x12),
            border: rgb(0x2A, 0x3A, 0x5A),
            text_primary: rgb(0xE2, 0xE8, 0xF0),
            text_muted: rgb(0x94, 0xA3, 0xB8),
            primary: rgb(0x2A, 0x6B, 0x9E),
            success: rgb(0x2E, 0xCC, 0x71),
            warning: rgb(0xF5, 0x9E, 0x0B),
            danger: rgb(0xEF, 0x44, 0x44),
        },
    };

    if let Some(accent) = custom_accent {
        base.primary = accent;
    }
    base
}

pub fn build_iced_theme(variant: ThemeVariant, custom_accent: Option<Color>) -> iced::Theme {
    let p = palette_for(variant, custom_accent);
    iced::Theme::custom(
        format!("Keeftalk {variant:?}"),
        Palette {
            background: p.background,
            text: p.text_primary,
            primary: p.primary,
            success: p.success,
            danger: p.danger,
        },
    )
}

pub fn parse_hex_color(hex: &str) -> Option<Color> {
    let clean = hex.trim().strip_prefix('#')?;
    if clean.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&clean[0..2], 16).ok()?;
    let g = u8::from_str_radix(&clean[2..4], 16).ok()?;
    let b = u8::from_str_radix(&clean[4..6], 16).ok()?;
    Some(rgb(r, g, b))
}

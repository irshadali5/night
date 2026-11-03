//! 🌙 Tokyo Night Theme Engine & Palette Specifications
//! Supports all 4 official variants: Night (Classic), Storm (Slate), Moon (Indigo), and Day (Light).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ThemeVariant {
    #[default]
    Night,
    Storm,
    Moon,
    Day,
}

impl ThemeVariant {
    pub const ALL: [ThemeVariant; 4] = [
        ThemeVariant::Night,
        ThemeVariant::Storm,
        ThemeVariant::Moon,
        ThemeVariant::Day,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Self::Night => "Night",
            Self::Storm => "Storm",
            Self::Moon => "Moon",
            Self::Day => "Day",
        }
    }

    pub fn is_dark(&self) -> bool {
        !matches!(self, Self::Day)
    }

    pub fn next(&self) -> Self {
        match self {
            Self::Night => Self::Storm,
            Self::Storm => Self::Moon,
            Self::Moon => Self::Day,
            Self::Day => Self::Night,
        }
    }

    pub fn parse_variant(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "night" => Some(Self::Night),
            "storm" => Some(Self::Storm),
            "moon" => Some(Self::Moon),
            "day" => Some(Self::Day),
            _ => None,
        }
    }

    pub fn palette(&self) -> Palette {
        match self {
            Self::Night => Palette {
                bg_darkest: "#16161e",
                bg_primary: "#1a1b26",
                bg_secondary: "#24283b",
                bg_highlight: "#414868",
                fg_primary: "#c0caf5",
                fg_muted: "#a9b1d6",
                fg_dark: "#565f89",
                accent_blue: "#7aa2f7",
                accent_cyan: "#7dcfff",
                accent_magenta: "#bb9af7",
                success_green: "#9ece6a",
                warning_yellow: "#e0af68",
                error_red: "#f7768e",
            },
            Self::Storm => Palette {
                bg_darkest: "#1f2335",
                bg_primary: "#24283b",
                bg_secondary: "#292e42",
                bg_highlight: "#414868",
                fg_primary: "#c0caf5",
                fg_muted: "#a9b1d6",
                fg_dark: "#565f89",
                accent_blue: "#7aa2f7",
                accent_cyan: "#7dcfff",
                accent_magenta: "#bb9af7",
                success_green: "#9ece6a",
                warning_yellow: "#e0af68",
                error_red: "#f7768e",
            },
            Self::Moon => Palette {
                bg_darkest: "#1e2030",
                bg_primary: "#222436",
                bg_secondary: "#2f334d",
                bg_highlight: "#444a73",
                fg_primary: "#c8d3f5",
                fg_muted: "#82aaff",
                fg_dark: "#636da6",
                accent_blue: "#82aaff",
                accent_cyan: "#86e1fc",
                accent_magenta: "#fca7ea",
                success_green: "#c3e88d",
                warning_yellow: "#ffc777",
                error_red: "#ff757f",
            },
            Self::Day => Palette {
                bg_darkest: "#d5d6db",
                bg_primary: "#e1e2e7",
                bg_secondary: "#e9e9ed",
                bg_highlight: "#c4c8da",
                fg_primary: "#3760bf",
                fg_muted: "#6172b0",
                fg_dark: "#8990b3",
                accent_blue: "#2e7de9",
                accent_cyan: "#007197",
                accent_magenta: "#9854f1",
                success_green: "#587539",
                warning_yellow: "#8c6c3e",
                error_red: "#f52a65",
            },
        }
    }
}

impl std::fmt::Display for ThemeVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    pub bg_darkest: &'static str,
    pub bg_primary: &'static str,
    pub bg_secondary: &'static str,
    pub bg_highlight: &'static str,
    pub fg_primary: &'static str,
    pub fg_muted: &'static str,
    pub fg_dark: &'static str,
    pub accent_blue: &'static str,
    pub accent_cyan: &'static str,
    pub accent_magenta: &'static str,
    pub success_green: &'static str,
    pub warning_yellow: &'static str,
    pub error_red: &'static str,
}

impl Palette {
    pub const BG_DARKEST: &'static str = "#16161e";
    pub const BG_PRIMARY: &'static str = "#1a1b26";
    pub const BG_SECONDARY: &'static str = "#24283b";
    pub const BG_HIGHLIGHT: &'static str = "#414868";

    pub const FG_PRIMARY: &'static str = "#c0caf5";
    pub const FG_MUTED: &'static str = "#a9b1d6";
    pub const FG_DARK: &'static str = "#565f89";

    // Accent Colors
    pub const ACCENT_BLUE: &'static str = "#7aa2f7";
    pub const ACCENT_CYAN: &'static str = "#7dcfff";
    pub const ACCENT_MAGENTA: &'static str = "#bb9af7";
    pub const SUCCESS_GREEN: &'static str = "#9ece6a";
    pub const WARNING_YELLOW: &'static str = "#e0af68";
    pub const ERROR_RED: &'static str = "#f7768e";
}

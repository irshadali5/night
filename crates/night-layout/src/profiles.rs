//! 🌙 Bihar Night Layout Profiles & Window Snapping Engine
//! Provides ergonomic workspace layout presets (Coding, Reading, Multitasking, Ultrawide)
//! and geometric snapping/centering for floating windows.

use crate::{request_relayout, set_layout_for_tag, state, LayoutKind};
use serde::{Deserialize, Serialize};
use tracing::info;

/// Predefined ergonomic layout profiles tailored for specific desktop workflows
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayoutProfile {
    /// Optimal for software engineering: Master & Stack (60/40 split), 8px inner gaps, 12px outer gaps
    Coding,
    /// Distraction-free reading/writing: Generous 32px outer margins, 12px inner gaps
    Reading,
    /// 4-quadrant multi-window dashboard: Fair equal-area grid, minimal 4px gaps
    Multitasking,
    /// Immersive ultrawide display layout: Fibonacci Spiral, borderless edge (0px outer gaps)
    Ultrawide,
    /// Single-window deep focus (Monocle, 0px gaps, 100% active surface)
    Focus,
}

impl LayoutProfile {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Coding => "Coding",
            Self::Reading => "Reading",
            Self::Multitasking => "Multitasking",
            Self::Ultrawide => "Ultrawide",
            Self::Focus => "Focus",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Coding => "MasterStack (60% master, 8px/12px gaps) for software development",
            Self::Reading => "MasterStack (32px generous margins) for documentation & study",
            Self::Multitasking => "Fair grid (4px minimal gaps) for multi-window dashboards",
            Self::Ultrawide => "Spiral (0px outer borderless) for ultrawide & media displays",
            Self::Focus => "Monocle (0px borderless, 100% focused window) for deep work",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Coding => "󰅩",
            Self::Reading => "󰈙",
            Self::Multitasking => "󱂬",
            Self::Ultrawide => "󰍹",
            Self::Focus => "󰊓",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "coding" | "code" | "dev" => Some(Self::Coding),
            "reading" | "read" | "doc" | "docs" => Some(Self::Reading),
            "multitasking" | "multi" | "dashboard" | "grid" => Some(Self::Multitasking),
            "ultrawide" | "wide" | "media" | "cinema" => Some(Self::Ultrawide),
            "focus" | "monocle" | "single" | "zen" => Some(Self::Focus),
            _ => None,
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Coding, Self::Reading, Self::Multitasking, Self::Ultrawide, Self::Focus]
    }

    /// Applies this layout profile to the active workspace tag
    pub fn apply(&self) {
        let active_tag = night_core::SessionState::load()
            .map(|s| s.active_tag)
            .unwrap_or_else(|| "1".to_string());

        let (layout, inner, outer, master_factor) = match self {
            Self::Coding => (LayoutKind::MasterStack, 8.0, 12.0, 0.60),
            Self::Reading => (LayoutKind::MasterStack, 12.0, 32.0, 0.50),
            Self::Multitasking => (LayoutKind::Fair, 4.0, 6.0, 0.50),
            Self::Ultrawide => (LayoutKind::Spiral, 6.0, 0.0, 0.55),
            Self::Focus => (LayoutKind::Monocle, 0.0, 0.0, 1.0),
        };

        {
            let mut guard = night_core::lock_unpoison(state());
            guard.inner_gaps = inner;
            guard.outer_gaps = outer;
            guard.master_factor = master_factor;
        }

        set_layout_for_tag(&active_tag, layout);
        info!(
            profile = %self.name(),
            tag = %active_tag,
            inner = inner,
            outer = outer,
            "Applied layout profile preset"
        );
        request_relayout();
    }
}

pub use crate::floating::SnapPosition;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_parsing() {
        assert_eq!(LayoutProfile::parse("coding"), Some(LayoutProfile::Coding));
        assert_eq!(LayoutProfile::parse("reading"), Some(LayoutProfile::Reading));
        assert_eq!(LayoutProfile::parse("grid"), Some(LayoutProfile::Multitasking));
        assert_eq!(LayoutProfile::parse("ultrawide"), Some(LayoutProfile::Ultrawide));
        assert_eq!(LayoutProfile::parse("focus"), Some(LayoutProfile::Focus));
        assert_eq!(LayoutProfile::parse("monocle"), Some(LayoutProfile::Focus));
        assert_eq!(LayoutProfile::parse("unknown_xyz"), None);
    }
}

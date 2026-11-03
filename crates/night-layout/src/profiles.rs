//! 🌙 Tokyo Night Layout Profiles & Window Snapping Engine
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
}

impl LayoutProfile {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Coding => "Coding",
            Self::Reading => "Reading",
            Self::Multitasking => "Multitasking",
            Self::Ultrawide => "Ultrawide",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Coding => "MasterStack (60% master, 8px/12px gaps) for software development",
            Self::Reading => "MasterStack (32px generous margins) for documentation & study",
            Self::Multitasking => "Fair grid (4px minimal gaps) for multi-window dashboards",
            Self::Ultrawide => "Spiral (0px outer borderless) for ultrawide & media displays",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Coding => "󰅩",
            Self::Reading => "󰈙",
            Self::Multitasking => "󱂬",
            Self::Ultrawide => "󰍹",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "coding" | "code" | "dev" => Some(Self::Coding),
            "reading" | "read" | "doc" | "docs" => Some(Self::Reading),
            "multitasking" | "multi" | "dashboard" | "grid" => Some(Self::Multitasking),
            "ultrawide" | "wide" | "media" | "cinema" => Some(Self::Ultrawide),
            _ => None,
        }
    }

    pub fn all() -> &'static [Self] {
        &[Self::Coding, Self::Reading, Self::Multitasking, Self::Ultrawide]
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
        };

        {
            let mut guard = state().lock().unwrap();
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

/// Snapping targets for floating client windows
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapPosition {
    Center,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    LeftHalf,
    RightHalf,
}

impl SnapPosition {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Center => "Center",
            Self::TopLeft => "Top-Left",
            Self::TopRight => "Top-Right",
            Self::BottomLeft => "Bottom-Left",
            Self::BottomRight => "Bottom-Right",
            Self::LeftHalf => "Left-Half",
            Self::RightHalf => "Right-Half",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_parsing() {
        assert_eq!(LayoutProfile::parse("coding"), Some(LayoutProfile::Coding));
        assert_eq!(LayoutProfile::parse("reading"), Some(LayoutProfile::Reading));
        assert_eq!(LayoutProfile::parse("grid"), Some(LayoutProfile::Multitasking));
        assert_eq!(LayoutProfile::parse("ultrawide"), Some(LayoutProfile::Ultrawide));
        assert_eq!(LayoutProfile::parse("unknown_xyz"), None);
    }
}

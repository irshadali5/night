//! 🌙 Night Layout Engine
//! Manages dynamic window tiling, layout switching (MasterStack, Dwindle, Spiral, Fair),
//! and interactive gaps for the Pinnacle compositor.

use night_core::UserConfig;
use pinnacle_api::layout::generators::{Dwindle, Fair, MasterSide, MasterStack, Spiral};
use pinnacle_api::layout::{Gaps, LayoutGenerator, LayoutRequester, LayoutResponse};
use pinnacle_api::util::Axis;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use tracing::info;

/// Available tiling layout algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayoutKind {
    /// Classic AwesomeWM Master & Stack
    MasterStack,
    /// Bspwm-style binary space partitioning
    Dwindle,
    /// Fibonacci golden spiral
    Spiral,
    /// Equal-area grid layout
    Fair,
}

impl LayoutKind {
    pub fn name(&self) -> &'static str {
        match self {
            Self::MasterStack => "MasterStack",
            Self::Dwindle => "Dwindle",
            Self::Spiral => "Spiral",
            Self::Fair => "Fair",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Self::MasterStack => Self::Dwindle,
            Self::Dwindle => Self::Spiral,
            Self::Spiral => Self::Fair,
            Self::Fair => Self::MasterStack,
        }
    }
}

pub struct LayoutState {
    pub default_kind: LayoutKind,
    pub tag_layouts: HashMap<String, LayoutKind>,
    pub inner_gaps: f32,
    pub outer_gaps: f32,
    pub master_factor: f32,
}

static LAYOUT_STATE: OnceLock<Arc<Mutex<LayoutState>>> = OnceLock::new();
static LAYOUT_REQUESTER: OnceLock<LayoutRequester> = OnceLock::new();

fn state() -> &'static Arc<Mutex<LayoutState>> {
    LAYOUT_STATE.get_or_init(|| {
        let user_cfg = UserConfig::load();
        Arc::new(Mutex::new(LayoutState {
            default_kind: LayoutKind::MasterStack,
            tag_layouts: HashMap::new(),
            inner_gaps: user_cfg.general.inner_gaps,
            outer_gaps: user_cfg.general.outer_gaps,
            master_factor: 0.55,
        }))
    })
}

/// Initializes dynamic layout management with Pinnacle
pub fn setup_layout() {
    info!("Initializing Night dynamic layout engine with per-tag memory...");

    let state_arc = state().clone();

    let requester = pinnacle_api::layout::manage(move |args| {
        let guard = state_arc.lock().unwrap();

        // Resolve active tag name if available
        let active_tag_name = args.tags.first().map(|t| t.name());
        let kind = if let Some(ref name) = active_tag_name {
            *guard.tag_layouts.get(name).unwrap_or(&guard.default_kind)
        } else {
            guard.default_kind
        };

        let inner = Gaps::from(guard.inner_gaps);
        let outer = Gaps::from(guard.outer_gaps);
        let master_factor = guard.master_factor;

        let root_node = match kind {
            LayoutKind::MasterStack => {
                let gen = MasterStack {
                    inner_gaps: inner,
                    outer_gaps: outer,
                    master_factor,
                    master_side: MasterSide::Left,
                    master_count: 1,
                    reversed: false,
                };
                gen.layout(args.window_count)
            }
            LayoutKind::Dwindle => {
                let gen = Dwindle {
                    inner_gaps: inner,
                    outer_gaps: outer,
                };
                gen.layout(args.window_count)
            }
            LayoutKind::Spiral => {
                let gen = Spiral {
                    inner_gaps: inner,
                    outer_gaps: outer,
                };
                gen.layout(args.window_count)
            }
            LayoutKind::Fair => {
                let gen = Fair {
                    inner_gaps: inner,
                    outer_gaps: outer,
                    axis: Axis::Vertical,
                };
                gen.layout(args.window_count)
            }
        };

        LayoutResponse {
            root_node,
            tree_id: kind as u32,
        }
    });

    let _ = LAYOUT_REQUESTER.set(requester);
    info!("Dynamic layout engine initialized with per-tag memory.");
}

/// Cycles to the next layout algorithm for the active tag and re-layouts the focused display
pub fn cycle_layout() -> LayoutKind {
    let active_tag = night_core::SessionState::load()
        .map(|s| s.active_tag)
        .unwrap_or_else(|| "1".to_string());

    let next = {
        let mut guard = state().lock().unwrap();
        let current = *guard.tag_layouts.get(&active_tag).unwrap_or(&guard.default_kind);
        let next = current.next();
        guard.tag_layouts.insert(active_tag.clone(), next);
        next
    };

    info!(tag = %active_tag, layout = %next.name(), "Cycled window layout for active workspace");
    request_relayout();
    next
}

/// Sets layout for a specific tag
pub fn set_layout_for_tag(tag: &str, layout: LayoutKind) {
    {
        let mut guard = state().lock().unwrap();
        guard.tag_layouts.insert(tag.to_string(), layout);
    }
    request_relayout();
}

/// Gets layout for a specific tag
pub fn get_layout_for_tag(tag: &str) -> LayoutKind {
    let guard = state().lock().unwrap();
    *guard.tag_layouts.get(tag).unwrap_or(&guard.default_kind)
}

/// Adjusts gaps in real-time
pub fn adjust_gaps(delta: f32) {
    {
        let mut guard = state().lock().unwrap();
        guard.inner_gaps = (guard.inner_gaps + delta).max(0.0);
        guard.outer_gaps = (guard.outer_gaps + delta).max(0.0);
        info!(inner = guard.inner_gaps, outer = guard.outer_gaps, "Updated layout gaps");
    }
    request_relayout();
}

/// Returns the current active layout kind
pub fn current_layout() -> LayoutKind {
    let active_tag = night_core::SessionState::load()
        .map(|s| s.active_tag)
        .unwrap_or_else(|| "1".to_string());
    let guard = state().lock().unwrap();
    *guard.tag_layouts.get(&active_tag).unwrap_or(&guard.default_kind)
}

/// Returns current (inner_gaps, outer_gaps)
pub fn get_gaps() -> (f32, f32) {
    let guard = state().lock().unwrap();
    (guard.inner_gaps, guard.outer_gaps)
}

/// Requests the compositor to recompute and apply the layout tree
pub fn request_relayout() {
    if let Some(req) = LAYOUT_REQUESTER.get() {
        req.request_layout();
    }
}

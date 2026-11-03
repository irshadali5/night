//! 🌙 Night Layout Engine
//! Manages dynamic window tiling, layout switching (MasterStack, Dwindle, Spiral, Fair),
//! and interactive gaps for the Pinnacle compositor.

use night_core::UserConfig;
use pinnacle_api::layout::generators::{Dwindle, Fair, MasterSide, MasterStack, Spiral};
use pinnacle_api::layout::{Gaps, LayoutGenerator, LayoutRequester, LayoutResponse};
use pinnacle_api::util::Axis;
use serde::{Deserialize, Serialize};
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
    pub current_kind: LayoutKind,
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
            current_kind: LayoutKind::MasterStack,
            inner_gaps: user_cfg.general.inner_gaps,
            outer_gaps: user_cfg.general.outer_gaps,
            master_factor: 0.55,
        }))
    })
}

/// Initializes dynamic layout management with Pinnacle
pub fn setup_layout() {
    info!("Initializing Night dynamic layout engine...");

    let state_arc = state().clone();

    let requester = pinnacle_api::layout::manage(move |args| {
        let guard = state_arc.lock().unwrap();
        let kind = guard.current_kind;
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
    info!("Dynamic layout engine initialized with MasterStack as default.");
}

/// Cycles to the next layout algorithm and re-layouts the focused display
pub fn cycle_layout() -> LayoutKind {
    let next = {
        let mut guard = state().lock().unwrap();
        guard.current_kind = guard.current_kind.next();
        guard.current_kind
    };

    info!(layout = %next.name(), "Cycled to next window layout");
    request_relayout();
    next
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
    state().lock().unwrap().current_kind
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

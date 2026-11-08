//! 🌙 Project Night: Window Focus Borders & Titlebar Decorations
//! Powered by Snowcap (`pinnacle_api::snowcap::FocusBorder`).
//! Provides Bihar Night theme-tailored focus frame accents, glowing floating window borders,
//! and optional native titlebars with close/maximize controls.

use night_core::theme::Palette;
use night_core::UserConfig;
use pinnacle_api::snowcap::FocusBorder;
use pinnacle_api::window::{self, WindowHandle};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::OnceLock;
use tracing::{debug, info};

static DECORATIONS_ENABLED: AtomicBool = AtomicBool::new(true);
static TITLEBARS_ENABLED: AtomicBool = AtomicBool::new(false);
static BORDER_WIDTH: AtomicU32 = AtomicU32::new(2);
static TITLEBAR_HEIGHT: AtomicU32 = AtomicU32::new(18);

/// Initializes window decoration and focus border hooks
pub fn setup_decorations() {
    let config = UserConfig::load();
    DECORATIONS_ENABLED.store(config.general.enable_decorations, Ordering::Relaxed);
    TITLEBARS_ENABLED.store(config.general.enable_titlebars, Ordering::Relaxed);
    BORDER_WIDTH.store(config.general.border_width.max(1), Ordering::Relaxed);
    TITLEBAR_HEIGHT.store(config.general.titlebar_height.max(14), Ordering::Relaxed);

    if !config.general.enable_decorations {
        info!("Window decorations disabled by configuration");
        return;
    }

    info!(
        border_width = config.general.border_width,
        titlebars = config.general.enable_titlebars,
        "Initializing Bihar Night Snowcap window decorations..."
    );

    // Decorate any existing windows
    for win in window::get_all() {
        decorate_window(&win, &config);
    }

    // Connect lifecycle signal for newly created windows
    static SIGNAL_CONNECTED: OnceLock<()> = OnceLock::new();
    SIGNAL_CONNECTED.get_or_init(|| {
        let handle = window::connect_signal(pinnacle_api::signal::WindowSignal::Created(Box::new(|win| {
            let cfg = UserConfig::load();
            decorate_window(win, &cfg);
        })));
        Box::leak(Box::new(handle));
    });
}

/// Decorates an individual window with a Bihar Night focus border and optional titlebar
pub fn decorate_window(window: &WindowHandle, config: &UserConfig) {
    if !DECORATIONS_ENABLED.load(Ordering::Relaxed) {
        return;
    }

    // Skip windows without foreign toplevel IDs (e.g. system surfaces)
    if window.foreign_toplevel_list_identifier().is_none() {
        return;
    }

    let palette = config.general.theme.palette();
    let width = BORDER_WIDTH.load(Ordering::Relaxed);
    let with_titlebar = TITLEBARS_ENABLED.load(Ordering::Relaxed);
    let tb_height = TITLEBAR_HEIGHT.load(Ordering::Relaxed);

    let mut border = if with_titlebar {
        let mut b = FocusBorder::new_with_titlebar(window);
        b.titlebar_height = tb_height;
        b
    } else {
        FocusBorder::new(window)
    };

    border.thickness = width;

    // Bihar Night focus border color selection
    // Floating windows receive an alluring lotus magenta glow
    let (f_r, f_g, f_b) = if window.floating() {
        Palette::hex_to_rgb(palette.accent_magenta)
    } else {
        Palette::hex_to_rgb(palette.accent_blue)
    };

    // Unfocused border matches background highlight
    let (u_r, u_g, u_b) = Palette::hex_to_rgb(palette.bg_highlight);

    border.focused_color = [f_r, f_g, f_b].into();
    border.unfocused_color = [u_r, u_g, u_b].into();

    match border.decorate() {
        Ok(handle) => {
            // Leak handle to keep decoration actively bound to window lifetime
            Box::leak(Box::new(handle));
            debug!(title = %window.title(), "Successfully decorated window");
        }
        Err(e) => {
            debug!(error = ?e, "Failed to attach FocusBorder decoration (window may have exited)");
        }
    }
}

/// Toggles window decorations on or off
pub fn toggle_decorations() -> bool {
    let cur = DECORATIONS_ENABLED.fetch_xor(true, Ordering::Relaxed);
    let new_state = !cur;
    info!(enabled = new_state, "Toggled window decorations");
    new_state
}

/// Toggles window titlebars on or off
pub fn toggle_titlebars() -> bool {
    let cur = TITLEBARS_ENABLED.fetch_xor(true, Ordering::Relaxed);
    let new_state = !cur;
    info!(titlebars = new_state, "Toggled window titlebars");
    new_state
}

/// Sets the window border width in pixels
pub fn set_border_width(width: u32) {
    let w = width.clamp(1, 16);
    BORDER_WIDTH.store(w, Ordering::Relaxed);
    info!(border_width = w, "Updated window border width");
}

//! 🌙 Tokyo Night Floating Window Snapping & Geometry Engine
//! Precise geometric calculations for screen-edge snapping, quadrant positioning,
//! half-splits, and picture-in-picture (PiP) mode.

use pinnacle_api::output::OutputHandle;
use pinnacle_api::window::WindowHandle;
use tracing::info;

pub use night_core::ipc::SnapPosition;

/// Applies geometric snapping to the given window on the target output monitor
pub fn snap_window(win: &WindowHandle, op: &OutputHandle, pos: SnapPosition) {
    let (inner_gaps, outer_gaps) = crate::get_gaps();
    let pad = outer_gaps.max(8.0) as i32;
    let half_gap = (inner_gaps.max(4.0) / 2.0) as i32;

    let op_size = op.logical_size().unwrap_or(pinnacle_api::util::Size {
        w: 1920,
        h: 1080,
    });
    let op_loc = op.loc().unwrap_or(pinnacle_api::util::Point { x: 0, y: 0 });

    let op_w = op_size.w as i32;
    let op_h = op_size.h as i32;

    // Ensure window is in floating mode
    win.set_floating(true);

    let (x, y, w, h) = match pos {
        SnapPosition::Center => {
            let win_size = win.size().unwrap_or(pinnacle_api::util::Size {
                w: (op_size.w * 7 / 10).max(800),
                h: (op_size.h * 7 / 10).max(520),
            });
            let target_w = (win_size.w as i32).min(op_w - pad * 2);
            let target_h = (win_size.h as i32).min(op_h - pad * 2);
            let target_x = op_loc.x + (op_w - target_w) / 2;
            let target_y = op_loc.y + (op_h - target_h) / 2;
            (target_x, target_y, target_w as u32, target_h as u32)
        }
        SnapPosition::LeftHalf => {
            let target_w = (op_w / 2) - pad - half_gap;
            let target_h = op_h - (pad * 2);
            let target_x = op_loc.x + pad;
            let target_y = op_loc.y + pad;
            (target_x, target_y, target_w.max(200) as u32, target_h.max(200) as u32)
        }
        SnapPosition::RightHalf => {
            let target_w = (op_w / 2) - pad - half_gap;
            let target_h = op_h - (pad * 2);
            let target_x = op_loc.x + (op_w / 2) + half_gap;
            let target_y = op_loc.y + pad;
            (target_x, target_y, target_w.max(200) as u32, target_h.max(200) as u32)
        }
        SnapPosition::TopHalf => {
            let target_w = op_w - (pad * 2);
            let target_h = (op_h / 2) - pad - half_gap;
            let target_x = op_loc.x + pad;
            let target_y = op_loc.y + pad;
            (target_x, target_y, target_w.max(200) as u32, target_h.max(200) as u32)
        }
        SnapPosition::BottomHalf => {
            let target_w = op_w - (pad * 2);
            let target_h = (op_h / 2) - pad - half_gap;
            let target_x = op_loc.x + pad;
            let target_y = op_loc.y + (op_h / 2) + half_gap;
            (target_x, target_y, target_w.max(200) as u32, target_h.max(200) as u32)
        }
        SnapPosition::TopLeft => {
            let target_w = (op_w / 2) - pad - half_gap;
            let target_h = (op_h / 2) - pad - half_gap;
            let target_x = op_loc.x + pad;
            let target_y = op_loc.y + pad;
            (target_x, target_y, target_w.max(200) as u32, target_h.max(200) as u32)
        }
        SnapPosition::TopRight => {
            let target_w = (op_w / 2) - pad - half_gap;
            let target_h = (op_h / 2) - pad - half_gap;
            let target_x = op_loc.x + (op_w / 2) + half_gap;
            let target_y = op_loc.y + pad;
            (target_x, target_y, target_w.max(200) as u32, target_h.max(200) as u32)
        }
        SnapPosition::BottomLeft => {
            let target_w = (op_w / 2) - pad - half_gap;
            let target_h = (op_h / 2) - pad - half_gap;
            let target_x = op_loc.x + pad;
            let target_y = op_loc.y + (op_h / 2) + half_gap;
            (target_x, target_y, target_w.max(200) as u32, target_h.max(200) as u32)
        }
        SnapPosition::BottomRight => {
            let target_w = (op_w / 2) - pad - half_gap;
            let target_h = (op_h / 2) - pad - half_gap;
            let target_x = op_loc.x + (op_w / 2) + half_gap;
            let target_y = op_loc.y + (op_h / 2) + half_gap;
            (target_x, target_y, target_w.max(200) as u32, target_h.max(200) as u32)
        }
        SnapPosition::Pip => {
            let pip_w = ((op_w * 32) / 100).max(420);
            let pip_h = ((pip_w * 9) / 16).max(240);
            let pip_pad = 24;
            let target_x = op_loc.x + op_w - pip_w - pip_pad;
            let target_y = op_loc.y + op_h - pip_h - pip_pad;

            // Make sticky across all workspace tags
            const DEFAULT_TAGS: &[&str] = &["1", "2", "3", "4", "5", "6", "7", "8", "9"];
            for name in DEFAULT_TAGS {
                if let Some(t) = pinnacle_api::tag::get_on_output(name, op) {
                    win.set_tag(&t, true);
                }
            }

            (target_x, target_y, pip_w as u32, pip_h as u32)
        }
        SnapPosition::Maximize => {
            let target_w = op_w - (pad * 2);
            let target_h = op_h - (pad * 2);
            let target_x = op_loc.x + pad;
            let target_y = op_loc.y + pad;
            (target_x, target_y, target_w.max(200) as u32, target_h.max(200) as u32)
        }
    };

    win.set_geometry(x, y, w, h);
    win.raise();

    info!(
        pos = %pos.name(),
        x = x,
        y = y,
        w = w,
        h = h,
        "Snapped floating window geometry"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snap_position_parse() {
        assert_eq!(SnapPosition::parse("center"), Some(SnapPosition::Center));
        assert_eq!(SnapPosition::parse("left"), Some(SnapPosition::LeftHalf));
        assert_eq!(SnapPosition::parse("right"), Some(SnapPosition::RightHalf));
        assert_eq!(SnapPosition::parse("top-left"), Some(SnapPosition::TopLeft));
        assert_eq!(SnapPosition::parse("pip"), Some(SnapPosition::Pip));
        assert_eq!(SnapPosition::parse("max"), Some(SnapPosition::Maximize));
        assert_eq!(SnapPosition::parse("invalid_pos"), None);
    }
}

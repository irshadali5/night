#![no_main]

use libfuzzer_sys::fuzz_target;
use night_core::ipc::SnapPosition;
use night_layout::{LayoutKind, LayoutProfile};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = LayoutKind::parse(s);
        let _ = LayoutProfile::parse(s);
        let _ = SnapPosition::parse(s);
    }
});

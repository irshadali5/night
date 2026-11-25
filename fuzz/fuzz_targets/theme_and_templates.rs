#![no_main]

use libfuzzer_sys::fuzz_target;
use night_core::clipboard::ClipboardHistory;
use night_core::ipc::DesktopStatus;
use night_core::notifications::NotificationHistory;
use night_core::state::SessionState;
use night_core::theme::{Palette, ThemeVariant};

fuzz_target!(|data: &[u8]| {
    let _ = postcard::from_bytes::<SessionState>(data);
    let _ = postcard::from_bytes::<ClipboardHistory>(data);
    let _ = postcard::from_bytes::<NotificationHistory>(data);

    if let Ok(s) = std::str::from_utf8(data) {
        let _ = ThemeVariant::parse_variant(s);
        let _ = Palette::hex_to_rgb(s);
        let _ = DesktopStatus::default().format_template(s);
    }
});

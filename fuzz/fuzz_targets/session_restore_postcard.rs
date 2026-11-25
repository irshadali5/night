#![no_main]

use libfuzzer_sys::fuzz_target;
use night_core::session_restore::{resolve_launch_command, SavedSession};

fuzz_target!(|data: &[u8]| {
    let _ = postcard::from_bytes::<SavedSession>(data);
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = resolve_launch_command(s);
    }
});

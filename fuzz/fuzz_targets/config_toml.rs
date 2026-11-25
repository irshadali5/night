#![no_main]

use libfuzzer_sys::fuzz_target;
use night_core::{MonitorProfile, UserConfig};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = toml::from_str::<UserConfig>(s);
        let _ = ron::from_str::<Vec<MonitorProfile>>(s);
    }
});

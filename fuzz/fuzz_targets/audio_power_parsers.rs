#![no_main]

use libfuzzer_sys::fuzz_target;
use night_core::audio::{parse_wpctl_sinks, parse_wpctl_volume};
use night_core::power::parse_battery_values;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = parse_wpctl_volume(s);
        let _ = parse_wpctl_sinks(s);
        let _ = parse_battery_values(s, Some(s));
    }
});

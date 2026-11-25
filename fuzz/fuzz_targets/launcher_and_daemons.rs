#![no_main]

use libfuzzer_sys::fuzz_target;
use night_binds::AppEntry;
use night_daemons::diurnal_watcher::resolve_diurnal_phase;

fuzz_target!(|data: &[u8]| {
    if data.len() >= 4 {
        let hour = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let _ = resolve_diurnal_phase(hour);
    }
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = AppEntry::clean_exec(s);
    }
});

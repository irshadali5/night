#![no_main]

use libfuzzer_sys::fuzz_target;
use night_rules::{rule_matches, swallow::is_terminal_app, Rule};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        if let Ok(rules) = ron::from_str::<Vec<Rule>>(s) {
            for r in &rules {
                let _ = rule_matches(r, s);
            }
        }
        let _ = is_terminal_app(s);
    }
});

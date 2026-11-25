//! 🌙 Night Window Rules
//! Defines auto-floating, window placement, and tag assignments for client windows using RON and standard XDG paths (`dirs`).

use pinnacle_api::{tag, window};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::{info, warn};

pub mod swallow;

/// Actions to apply to matched windows
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WindowAction {
    /// Float the window
    Float,
    /// Send window to a specific tag
    MoveToTag(String),
    /// Swallows the focused parent terminal window
    Swallow,
}

/// A declarative window rule defined in RON
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Rule {
    pub app_id: String,
    pub action: WindowAction,
}

pub fn default_rules() -> Vec<Rule> {
    vec![
        Rule {
            app_id: "night-scratchpad".to_string(),
            action: WindowAction::Float,
        },
        Rule {
            app_id: "satty".to_string(),
            action: WindowAction::Float,
        },
        Rule {
            app_id: "pavucontrol".to_string(),
            action: WindowAction::Float,
        },
        Rule {
            app_id: "night-settings".to_string(),
            action: WindowAction::Float,
        },
        Rule {
            app_id: "nm-connection-editor".to_string(),
            action: WindowAction::Float,
        },
        Rule {
            app_id: "zen".to_string(),
            action: WindowAction::MoveToTag("2".to_string()),
        },
        Rule {
            app_id: "brave".to_string(),
            action: WindowAction::MoveToTag("2".to_string()),
        },
        Rule {
            app_id: "discord".to_string(),
            action: WindowAction::MoveToTag("8".to_string()),
        },
        Rule {
            app_id: "spotify".to_string(),
            action: WindowAction::MoveToTag("9".to_string()),
        },
    ]
}

/// Resolves standard XDG path via `dirs`
pub fn rules_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("night/rules.ron")
}

/// Loads rules from standard XDG path or defaults
pub fn load_rules() -> Vec<Rule> {
    let path = rules_path();

    if path.exists() {
        match fs::read_to_string(&path) {
            Ok(content) => match ron::from_str::<Vec<Rule>>(&content) {
                Ok(rules) => {
                    info!(count = rules.len(), path = %path.display(), "Loaded window rules via RON");
                    return rules;
                }
                Err(e) => {
                    warn!(error = %e, "Failed to parse rules.ron, using default rules");
                }
            },
            Err(e) => {
                warn!(error = %e, "Failed to read rules.ron, using default rules");
            }
        }
    }

    default_rules()
}

/// Initializes window rule monitoring
pub fn setup_rules() {
    let rules = load_rules();
    info!(count = rules.len(), "Initializing declarative window rules...");

    // Initialize terminal window swallowing engine
    swallow::setup_swallow_listeners();

    window::add_window_rule(move |window| {
        let app_id = window.app_id().to_lowercase();

        for rule in &rules {
            if app_id == rule.app_id.to_lowercase() {
                match &rule.action {
                    WindowAction::Float => {
                        window.set_floating(true);
                    }
                    WindowAction::MoveToTag(tag_name) => {
                        if let Some(t) = tag::get(tag_name) {
                            window.move_to_tag(&t);
                        }
                    }
                    WindowAction::Swallow => {
                        if let Some(focused) = window::get_focused() {
                            if focused.id() != window.id() && swallow::is_terminal_app(&focused.app_id()) {
                                swallow::swallow(&window, &focused);
                            }
                        }
                    }
                }
            }
        }
    });

    info!("Window rules and swallowing engine active.");
}

/// Evaluates whether a [`Rule`] matches the given window `app_id` (case-insensitive).
pub fn rule_matches(rule: &Rule, app_id: &str) -> bool {
    rule.app_id.eq_ignore_ascii_case(app_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_rules_and_matching() {
        let rules = default_rules();
        assert!(!rules.is_empty());
        let satty = rules.iter().find(|r| rule_matches(r, "Satty")).unwrap();
        assert_eq!(satty.action, WindowAction::Float);

        let zen = rules.iter().find(|r| rule_matches(r, "zen")).unwrap();
        assert_eq!(zen.action, WindowAction::MoveToTag("2".to_string()));
    }

    #[test]
    fn test_rules_ron_roundtrip() {
        let rules = default_rules();
        let ron_str = ron::to_string(&rules).expect("serialize rules");
        let parsed: Vec<Rule> = ron::from_str(&ron_str).expect("deserialize rules");
        assert_eq!(parsed, rules);
    }

    #[test]
    fn test_fuzz_smoke_rules_and_swallow() {
        for sample in ["", "[]", "[(app_id: \"x\", action: Float)]", "bad ron", "\0\x7f"] {
            let _ = ron::from_str::<Vec<Rule>>(sample);
            let _ = swallow::is_terminal_app(sample);
        }
    }
}

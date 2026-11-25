//! 🌙 Bihar Night Application Launcher & Command Runner Modal
//! Fast, pure-Rust XDG desktop application scanner and layer-shell runner modal.
//! Supports instant single-key launch (`[1-8]`), mnemonic shortcuts (`[T]`, `[M]`, `[F]`, `[B]`, `[S]`),
//! and fallback to extended search (`[9]` Walker).

use pinnacle_api::experimental::snowcap_api::layer::{
    new_widget, ExclusiveZone, KeyboardInteractivity, ZLayer,
};
use pinnacle_api::experimental::snowcap_api::widget::{
    column::Column,
    container::Container,
    font::{Family, Font, Weight},
    text::{self, Text},
    Alignment, Background, Length, Padding, WidgetDef,
};
use pinnacle_api::Keysym;
use night_core::exec::{resolve_executable, spawn_shell};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;

/// An installed desktop application entry parsed from XDG `.desktop` files
#[derive(Clone, Debug)]
pub struct AppEntry {
    pub name: String,
    pub exec: String,
    pub comment: String,
    pub icon: String,
    pub terminal: bool,
}

impl AppEntry {
    /// Cleans command strings by removing XDG field codes (%f, %u, %F, %U, etc.)
    pub fn clean_exec(raw: &str) -> String {
        let mut parts = Vec::new();
        for token in raw.split_whitespace() {
            if token.starts_with('%') {
                continue;
            }
            parts.push(token);
        }
        parts.join(" ")
    }
}

/// Scans standard XDG directories and returns discovered applications
pub fn scan_desktop_applications() -> Vec<AppEntry> {
    let mut search_dirs: Vec<PathBuf> = Vec::new();

    if let Some(user_apps) = dirs::data_dir().map(|d| d.join("applications")) {
        if user_apps.is_dir() {
            search_dirs.push(user_apps);
        }
    }

    let sys_apps = Path::new("/usr/share/applications");
    if sys_apps.is_dir() {
        search_dirs.push(sys_apps.to_path_buf());
    }

    let local_apps = Path::new("/usr/local/share/applications");
    if local_apps.is_dir() {
        search_dirs.push(local_apps.to_path_buf());
    }

    let mut apps: Vec<AppEntry> = Vec::new();
    let mut seen_names: std::collections::HashSet<String> = std::collections::HashSet::new();

    for dir in search_dirs {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("desktop") {
                continue;
            }

            if let Ok(content) = fs::read_to_string(&path) {
                if let Some(app) = parse_desktop_entry(&content) {
                    let key = app.name.to_lowercase();
                    if !seen_names.contains(&key) {
                        seen_names.insert(key);
                        apps.push(app);
                    }
                }
            }
        }
    }

    // Ensure our signature Bihar Night desktop suite applications are prioritized if installed
    let mut prioritized = Vec::new();
    let priority_names = [
        ("Alacritty", "alacritty", "GPU-accelerated terminal emulator", false),
        ("Rio", "rio", "Modern WebGPU-powered terminal", false),
        ("Vivi", "vivi", "Rust & Slint pure-native media player", false),
        ("Favnyr", "favnyr", "Modern Rust + Slint GUI file manager", false),
        ("Yazi", "yazi", "Lightning-fast terminal file manager", true),
        ("Zen Browser", "zen-browser", "Modern privacy-focused web browser", false),
        ("Brave", "brave", "Fast, private Chromium-based browser", false),
        ("Night Settings", "night-settings", "Pure-Rust Slint DE Control Center", false),
    ];

    for (name, bin, comment, is_term) in priority_names {
        if resolve_executable(bin).is_some() {
            if let Some(idx) = apps.iter().position(|a| a.name.eq_ignore_ascii_case(name)) {
                let app = apps.remove(idx);
                prioritized.push(app);
            } else {
                prioritized.push(AppEntry {
                    name: name.to_string(),
                    exec: bin.to_string(),
                    comment: comment.to_string(),
                    icon: bin.to_string(),
                    terminal: is_term,
                });
            }
        }
    }

    // Sort remaining apps alphabetically
    apps.sort_by_key(|a| a.name.to_lowercase());
    prioritized.extend(apps);
    prioritized
}

/// Parses a `.desktop` file content
fn parse_desktop_entry(content: &str) -> Option<AppEntry> {
    let mut in_desktop_entry = false;
    let mut name = String::new();
    let mut exec = String::new();
    let mut comment = String::new();
    let mut icon = String::new();
    let mut terminal = false;
    let mut nodisplay = false;
    let mut is_app = true;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_desktop_entry = trimmed == "[Desktop Entry]";
            continue;
        }

        if !in_desktop_entry || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if let Some((k, v)) = trimmed.split_once('=') {
            let key = k.trim();
            let val = v.trim();

            match key {
                "Type" => {
                    if val != "Application" {
                        is_app = false;
                    }
                }
                "Name" if name.is_empty() => {
                    name = val.to_string();
                }
                "Exec" if exec.is_empty() => {
                    exec = AppEntry::clean_exec(val);
                }
                "Comment" if comment.is_empty() => {
                    comment = val.to_string();
                }
                "Icon" if icon.is_empty() => {
                    icon = val.to_string();
                }
                "Terminal" => {
                    terminal = val.eq_ignore_ascii_case("true");
                }
                "NoDisplay" | "Hidden" if val.eq_ignore_ascii_case("true") => {
                    nodisplay = true;
                }
                _ => {}
            }
        }
    }

    if !is_app || nodisplay || name.is_empty() || exec.is_empty() {
        return None;
    }

    Some(AppEntry {
        name,
        exec,
        comment,
        icon,
        terminal,
    })
}

/// Layer-shell App Launcher Modal
#[derive(Clone, Debug)]
pub struct LauncherModal {
    pub border_radius: f32,
    pub border_thickness: f32,
    pub background_color: [f32; 4],
    pub border_color: [f32; 4],
    pub font: Font,
    pub width: u32,
    pub height: u32,
    pub apps: Vec<AppEntry>,
}

impl Default for LauncherModal {
    fn default() -> Self {
        Self {
            border_radius: 14.0,
            border_thickness: 2.0,
            background_color: [0.10, 0.11, 0.15, 0.96], // Bihar Night Dark #1a1b26
            border_color: [0.48, 0.64, 0.97, 1.0],     // Bihar Night Blue #7aa2f7
            font: Font::new_with_family(Family::Name("Ubuntu".into())),
            width: 720,
            height: 440,
            apps: Vec::new(),
        }
    }
}

impl pinnacle_api::experimental::snowcap_api::widget::Program for LauncherModal {
    type Message = ();

    fn update(&mut self, _msg: Self::Message) {}

    fn view(&self) -> Option<WidgetDef<Self::Message>> {
        let title = Text::new("󰀻 Bihar Night Application Launcher & Runner").style(
            text::Style::new()
                .font(self.font.clone().weight(Weight::Bold))
                .pixels(18.0),
        );

        let subtitle = Text::new(
            "Press [1-8] or Mnemonic to Launch  •  [9] Walker Search  •  [Esc] Cancel",
        )
        .style(text::Style::new().font(self.font.clone()).pixels(12.0));

        let mut children: Vec<WidgetDef<Self::Message>> = vec![
            title.into(),
            Text::new("").style(text::Style::new().pixels(4.0)).into(),
            subtitle.into(),
            Text::new("").style(text::Style::new().pixels(12.0)).into(),
        ];

        // Display top 8 applications
        for (idx, app) in self.apps.iter().take(8).enumerate() {
            let num = idx + 1;
            let comment_preview = if app.comment.is_empty() {
                format!("command: {}", app.exec)
            } else if app.comment.chars().count() > 36 {
                let s: String = app.comment.chars().take(33).collect();
                format!("{s}...")
            } else {
                app.comment.clone()
            };

            let line = format!(
                "  [{num}] 󰀻  {:<24}  •  {:<36}",
                app.name, comment_preview
            );

            let row_text = Text::new(line).style(
                text::Style::new()
                    .font(self.font.clone())
                    .pixels(13.0),
            );

            children.push(row_text.into());
            children.push(Text::new("").style(text::Style::new().pixels(4.0)).into());
        }

        // Slot [9]: Extended Launcher (Walker)
        let slot9_line = "  [9] 󰍉  Extended Launcher (Walker)  •  Fuzzy search all system binaries & apps".to_string();
        let slot9_text = Text::new(slot9_line).style(
            text::Style::new()
                .font(self.font.clone().weight(Weight::Bold))
                .pixels(13.0),
        );
        children.push(slot9_text.into());
        children.push(Text::new("").style(text::Style::new().pixels(10.0)).into());

        // Quick Mnemonics bar
        let mnemonics = "Quick Keys: [T] Terminal   [M] Vivi Media   [F] Favnyr FM   [B] Browser   [S] Settings";
        let mnemonics_text = Text::new(mnemonics).style(
            text::Style::new()
                .font(self.font.clone())
                .pixels(11.0),
        );
        children.push(mnemonics_text.into());

        let widget = Container::new(Column::new_with_children(children))
            .width(Length::Fixed(self.width as f32))
            .height(Length::Fixed(self.height as f32))
            .padding(Padding {
                top: 20.0,
                right: 24.0,
                bottom: 20.0,
                left: 24.0,
            })
            .vertical_alignment(Alignment::Start)
            .horizontal_alignment(Alignment::Start)
            .style(
                pinnacle_api::experimental::snowcap_api::widget::container::Style {
                    text_color: None,
                    background: Some(Background::Color(self.background_color.into())),
                    border: Some(pinnacle_api::experimental::snowcap_api::widget::Border {
                        color: Some(self.border_color.into()),
                        width: Some(self.border_thickness),
                        radius: Some(self.border_radius.into()),
                    }),
                },
            );

        Some(widget.into())
    }
}

/// Spawns a shell or application command safely
fn launch_app(app: &AppEntry) {
    let term = night_core::config::UserConfig::load().apps.terminal;
    let cmd = if app.terminal {
        format!("{term} -e {}", app.exec)
    } else {
        app.exec.clone()
    };
    info!(name = %app.name, cmd = %cmd, "Launching application from modal");
    spawn_shell(&cmd);
}

/// Displays the interactive Bihar Night Application Launcher Modal via Snowcap
pub fn show_launcher_modal() {
    let apps = scan_desktop_applications();
    let apps_clone = apps.clone();

    let modal = LauncherModal {
        apps,
        ..Default::default()
    };

    let _ = new_widget(
        modal,
        None,
        KeyboardInteractivity::Exclusive,
        ExclusiveZone::Respect,
        ZLayer::Overlay,
    )
    .map(|handle| {
        handle.on_key_press(move |h, key, _mods| {
            let select_idx = match key {
                Keysym::_1 => Some(0),
                Keysym::_2 => Some(1),
                Keysym::_3 => Some(2),
                Keysym::_4 => Some(3),
                Keysym::_5 => Some(4),
                Keysym::_6 => Some(5),
                Keysym::_7 => Some(6),
                Keysym::_8 => Some(7),
                Keysym::Return => Some(0),
                // [9] Extended launcher (Walker)
                Keysym::_9 => {
                    h.close();
                    let launcher = night_core::config::UserConfig::load().apps.launcher;
                    spawn_shell(&launcher);
                    return;
                }
                // Mnemonic shortcuts
                Keysym::t | Keysym::T => {
                    h.close();
                    let term = night_core::config::UserConfig::load().apps.terminal;
                    spawn_shell(&term);
                    return;
                }
                Keysym::m | Keysym::M => {
                    h.close();
                    let media = night_core::config::UserConfig::load().apps.media_player;
                    spawn_shell(&media);
                    return;
                }
                Keysym::f | Keysym::F => {
                    h.close();
                    let fm = night_core::config::UserConfig::load().apps.file_manager;
                    spawn_shell(&fm);
                    return;
                }
                Keysym::b | Keysym::B => {
                    h.close();
                    let browser = night_core::config::UserConfig::load().apps.browser;
                    spawn_shell(&browser);
                    return;
                }
                Keysym::s | Keysym::S => {
                    h.close();
                    spawn_shell("night-settings");
                    return;
                }
                _ => {
                    h.close();
                    return;
                }
            };

            if let Some(idx) = select_idx {
                if let Some(app) = apps_clone.get(idx) {
                    launch_app(app);
                }
            }
            h.close();
        });
    });
}

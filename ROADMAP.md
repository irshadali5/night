# 🌙 Project Night: Roadmap

> A high-performance, minimalist, 100% Rust-native Wayland desktop environment powered by [Pinnacle](https://github.com/pinnacle-comp/pinnacle).

---

## 🎯 Vision & Goals
* **Memory Safety & Speed:** Entire desktop stack built on Rust (Compositor, Bars, Widgets, Launchers, Daemons, OSD, Wallpapers, Terminals, File Manager).
* **AwesomeWM Workflow on Wayland:** Tag-based dynamic tiling with deep, programmatic Rust configuration via `pinnacle-api`.
* **Idiomatic Cargo Workspace:** Clean 7-crate root workspace (`crates/`) with native `cargo xtask` runner.
* **Purpose-Driven Formats:** 
  * `TOML` for human-editable settings.
  * `RON` for strongly-typed Rust window rules and enums.
  * `postcard` for instant zero-allocation binary session caching.
  * `xtask` for pure-Rust workspace task automation.
* **Separation of Concerns:** Night repository contains the Rust orchestration crates; external tools are installed as standalone packages and coordinated by Night.
* **Late-Night Ergonomics:** Low blue light, low visual noise, unified Tokyo Night dark palette, keyboard-driven navigation.

---

## 🗺️ Phases Overview

```
Phase 1 ──> ... ──> Phase 22 ──> Phase 23 ──> Phase 24 ──> Modernization
[Workspace]         [Session]   [Snapping/PiP] [Borders/Titlebars] [Active Open Crates]
```

---

### Phase 1: Compositor Core Workspace Bootstrap (Pinnacle + Rust)
* [x] Initialize repository roadmap and implementation specifications.
* [x] Setup root-level 7-crate **Cargo Workspace**:
  * [x] **`xtask`**: Native task runner (`cargo xtask`).
  * [x] **`night-core`**: Theme palette constants, default apps, async process/shell execution.
  * [x] **`night-daemons`**: Background daemon specifications and supervisor.
  * [x] **`night-tags`**: AwesomeWM-style dynamic tag management and switching.
  * [x] **`night-rules`**: Declarative client window floating and tag placement rules.
  * [x] **`night-binds`**: Centralized keybindings (launchers, terminals, OSD, screenshots).
  * [x] **`night-pinnacle`**: Binary entry point invoking `pinnacle_api::main!(config)`.
* [x] Create root `pinnacle.toml` runtime manifest.

---

### Phase 2: Format Specialization (TOML, RON, postcard)
* [x] **TOML Configuration**:
  * [x] Create `config/night.toml` user configuration schema.
  * [x] Implement `night_core::config::UserConfig::load()` with defaults fallback.
* [x] **RON Window Rules**:
  * [x] Create `config/rules.ron` with typed Rust enums (`Float`, `MoveToTag`).
  * [x] Implement `night_rules::load_rules()` using `ron::from_str`.
* [x] **Postcard Binary Session Caching**:
  * [x] Implement `night_core::state::SessionState` using `postcard`.
  * [x] Save and restore session state in `$XDG_RUNTIME_DIR/night-session.bin`.

---

### Phase 3: Notifications, OSD, Terminals & Multimedia Orchestration
* [x] Wire up daemon startup sequence in `night-daemons` (`start_all()`).
* [x] Map hardware multimedia keys (`XF86Audio*`, `XF86MonBrightness*`) to SwayOSD in `night-binds`.
* [x] Connect screenshots (`Wayshot` + `Satty`) and screen recording (`wl-screenrec`) in `night-binds`.
* [x] Connect terminal and file manager launching in `night-binds`.

---

### Phase 4: `cargo xtask` Automation
* [x] Eliminate Stow, Makefiles, and shell scripts.
* [x] Implement `cargo xtask install` (build, copy to `~/.cargo/bin`, symlink `pinnacle.toml`, init `~/.config/night/`).
* [x] Implement `cargo xtask check`, `cargo xtask build`, `cargo xtask run`, `cargo xtask uninstall`.
* [x] Configure `.cargo/config.toml` alias for `cargo xtask`.

---

### Phase 5: Multi-Monitor, Interactive Navigation & IPC Hardening
* [x] Unified Tokyo Night dark color palette in `night-core::theme`.
* [x] **Multi-Monitor Hotplugging (`night-tags`)**:
  * [x] Dynamic monitor detection via `output::for_each_output` for live display hotplugging.
  * [x] Automated workspace tag provisioning across all connected displays.
* [x] **Interactive Window & Mouse Navigation (`night-binds`)**:
  * [x] Vim-style directional focus (`Super + h/j/k/l`) and Arrow keys via `in_direction`.
  * [x] Window swapping (`Super + Shift + h/j/k/l`) with adjacent neighbors.
  * [x] Fullscreen toggle (`Super + f`) and floating mode toggle (`Super + Shift + Space`).
  * [x] Interactive floating window drag (`Super + LeftClick`) via `window::begin_move`.
  * [x] Interactive floating window resize (`Super + RightClick`) via `window::begin_resize`.
* [x] **Idle & Session Lock Management (`night-daemons`)**:
  * [x] Dynamic session locking (`Super + Escape`) using **Veila** (pure-Rust Wayland locker).
  * [x] Automated idle timeout trigger via `wayidle` using configured `lock_cmd`.
* [x] **Desktop Interprocess Communication (IPC)**:
  * [x] Async Unix socket server in `night-pinnacle` (`/run/user/1000/night.sock`).
  * [x] Binary serialized IPC commands (`ReloadConfig`, `SwitchTag`, `QueryState`, `Spawn`, `CloseFocused`) via `postcard`.
  * [x] CLI control runner in task runner (`cargo xtask msg <reload|switch-tag|state|spawn|close>`).

---

### Phase 6: Dynamic Layout Engine & Tokyo Night Desktop Suite Exporter
* [x] **Dynamic Layout Engine (`crates/night-layout`)**:
  * [x] Created `night-layout` crate implementing `pinnacle_api::layout::manage`.
  * [x] State machine supporting 4 dynamic tiling algorithms: `MasterStack`, `Dwindle`, `Spiral`, and `Fair`.
  * [x] Real-time gaps adjustment (`inner_gaps` and `outer_gaps`) via atomic state and `LayoutRequester`.
* [x] **Keybindings for Layout & Gaps (`night-binds`)**:
  * [x] `Super + Tab` $\rightarrow$ Cycle through layouts (`MasterStack` $\rightarrow$ `Dwindle` $\rightarrow$ `Spiral` $\rightarrow$ `Fair`).
  * [x] `Super + equal` $\rightarrow$ Increase gaps (+2px).
  * [x] `Super + minus` $\rightarrow$ Decrease gaps (-2px).
* [x] **IPC Layout Remote Control (`night-core::ipc`)**:
  * [x] Added `CycleLayout`, `AdjustGaps`, and `GetLayout` IPC commands.
  * [x] `cargo xtask msg layout [next]` and `cargo xtask msg gaps <delta>`.
* [x] **Tokyo Night Theme Exporter (`xtask theme`)**:
  * [x] Pure-Rust theme generator in `cargo xtask theme`.
  * [x] Generates matching configurations for external suite: `alacritty.toml`, `ironbar.css`, `anyrun.css`, `veila.toml`.
  * [x] Automatically deployed to user config paths on `cargo xtask install`.

---

### Phase 7: Compositor Lifecycle Signals, System Doctor & Session Packaging
* [x] **Real-time Signal Integration (`pinnacle_api::signal`)**:
  * [x] Hooked `TagSignal::Active` in `night-tags` to continuously synchronize binary session state on tag switch events.
* [x] **Desktop Environment & Suite Doctor (`cargo xtask doctor`)**:
  * [x] Pure-Rust diagnostic tool checking presence of all 18 tools in the 100% Rust-native desktop suite.
  * [x] Validates configuration paths and Wayland session environment variables (`WAYLAND_DISPLAY`, `XDG_SESSION_TYPE`).
  * [x] Emits consolidated, copy-pasteable Arch Linux `yay -S --needed ...` command for missing packages.
* [x] **Wayland Session Packaging & Launcher (`assets/`)**:
  * [x] Created `assets/night.desktop` for login display managers (GDM, SDDM, Ly, greetd).
  * [x] Created `assets/night-session` launcher script with pure-Wayland toolkit environment variables.
  * [x] Implemented `cargo xtask session` to install session files to `~/.cargo/bin` and wayland-sessions.
* [x] **Comprehensive Status Reporting (`cargo xtask msg status`)**:
  * [x] Added `GetStatus` IPC query returning `DesktopStatus` (active tag, layout, inner/outer gaps, connected monitor count, focused window title).

---

### Phase 8: Per-Tag Layout Memory, Dropdown Scratchpad & Ironbar TOML Integration
* [x] **Per-Tag Dynamic Layout Memory (`crates/night-layout`)**:
  * [x] Implemented per-tag layout state (`HashMap<String, LayoutKind>`).
  * [x] Each workspace tag (1..9) independently remembers its preferred tiling algorithm across switches (AwesomeWM style).
* [x] **Quake-Style Dropdown Scratchpad Terminal (`night-binds`, `night-rules`, `config/rules.ron`)**:
  * [x] Bound to `Super + grave` (tilde `~` key) for instant toggle.
  * [x] Window tag visibility management: hides when visible on active tag; attaches, raises, and focuses when hidden.
  * [x] Auto-floats via declarative window matching rules in `rules.ron`.
  * [x] Remote triggerable via IPC (`cargo xtask msg scratchpad`).
* [x] **Ironbar Native TOML Configuration (`config/ironbar.toml`)**:
  * [x] Created clean TOML configuration for Ironbar GTK4-layer-shell status bar.
  * [x] Configured modules: `workspaces`, `focused` window title, `clock`, `sys_info`, `volume`, and `tray`.
  * [x] Automated deployment to `~/.config/ironbar/config.toml` alongside `style.css`.

---

### Phase 9: Real-time Bar Integration, Snowcap Companion Widgets & `nightctl` CLI Client
* [x] **Dedicated CLI Binary (`crates/night-ctl`)**:
  * [x] Created `nightctl` binary crate for sub-millisecond local command execution (`tag`, `layout`, `gaps`, `scratchpad`, `overlay`, `quit-prompt`, `status`, `stream`).
  * [x] Added `stream` mode for streaming real-time status updates without polling lag (`--json` or `--format "{layout} | {active_tag} | {active_window}"`).
  * [x] Integrated into workspace members and automated installation via `cargo xtask install` into `~/.cargo/bin/nightctl`.
* [x] **Multi-Client Pub/Sub IPC Streaming (`crates/night-core/src/ipc.rs`)**:
  * [x] Transitioned IPC server to async multi-client task spawning with `tokio::sync::broadcast`.
  * [x] Added `IpcCommand::Subscribe` persistent event stream delivering zero-allocation postcard binary updates.
  * [x] Connected compositor signals (`TagSignal::Active`, `WindowSignal::Focused`, `WindowSignal::TitleChanged`) for live state synchronization.
* [x] **Native Snowcap Widget Integration (`crates/night-binds`)**:
  * [x] Wired up `pinnacle_api::snowcap::BindOverlay` (`Super + /`) displaying all registered keybindings in a Tokyo Night styled modal.
  * [x] Wired up `pinnacle_api::snowcap::QuitPrompt` (`Super + Shift + e`) with confirmation and Tokyo Night dark palette styling.
  * [x] Exposed remote triggers via IPC (`IpcCommand::ShowBinds`, `IpcCommand::ShowQuit`).
* [x] **Ironbar Real-Time Layout Stream Module (`config/ironbar.toml`)**:
  * [x] Added custom layout module running `nightctl stream --format '󰕰 {layout}'` with interactive click-to-cycle (`on_click_left = "nightctl layout next"`).

---

### Phase 10: Dynamic Wallpaper Transitions, Screen Capture, Systemd Units & Arch Linux PKGBUILD
* [x] **Dynamic Wallpaper Controller (`assets/wallpaper.sh`)**:
  * [x] Pure-Rust Wayland wallpaper manager using `swww` with smooth Tokyo Night transition shaders (`--transition-type grow`).
  * [x] Automatic discovery from `~/.config/night/wallpapers/` or `~/Pictures/Wallpapers/` with fallback to solid Tokyo Night canvas (`#1a1b26`).
* [x] **Rust-Native Screen Capture & Recording Helper Scripts (`assets/`)**:
  * [x] `assets/screenshot.sh`: Pure-Rust interactive selection and fullscreen capture via `wayshot` + `slurp` + `satty`.
  * [x] `assets/record.sh`: Hardware-accelerated Wayland screen recorder toggle via `wl-screenrec`.
* [x] **Systemd User Session Lifecycle Integration (`assets/`)**:
  * [x] Created `assets/night-session.target` and `assets/night-session.service` for systemd `--user` process supervision.
* [x] **Arch Linux & AUR PKGBUILD Specification (`packaging/PKGBUILD`)**:
  * [x] Complete Arch Linux packaging specification compiling `night-pinnacle` and `nightctl` with `--release --locked`.
  * [x] Installs binaries, session desktop entry, scripts, Tokyo Night config templates, and systemd units to standard Arch Linux paths.

---

### Phase 11: Multi-Monitor Per-Output Tag Isolation, Monitor Profiles & DPMS Power Management
* [x] **Declarative RON Monitor Profiles (`config/monitors.ron` & `crates/night-core/src/monitors.rs`)**:
  * [x] Connector-based display profile schema defining HiDPI scale factor, mode (resolution & refresh rate), position coordinates, transform orientation, and VRR adaptive sync.
  * [x] Automatic hotplug application in `night-tags` on output discovery and connection signals.
* [x] **Per-Output Workspace Tag Isolation (`crates/night-tags`)**:
  * [x] Implemented independent workspace tags `[1..9]` per physical monitor connector (AwesomeWM/XMonad style).
  * [x] `switch_to_tag` and `move_focused_to_tag` now operate on the focused monitor via `tag::get_on_output`.
* [x] **Multi-Monitor Navigation & Window Movement (`crates/night-binds` & `nightctl`)**:
  * [x] `Super + o` / `Super + period` $\rightarrow$ Focus next display output.
  * [x] `Super + comma` $\rightarrow$ Focus previous display output.
  * [x] `Super + Shift + o` / `Super + Shift + period` $\rightarrow$ Move focused window to next display output.
  * [x] `Super + Shift + comma` $\rightarrow$ Move focused window to previous display output.
  * [x] `Super + Shift + Escape` $\rightarrow$ Toggle DPMS display power across all screens.
* [x] **CLI Remote Control & IPC Integration (`nightctl output` & `nightctl dpms`)**:
  * [x] Added `output <next|prev|focus|move-window>` and `dpms [on|off|toggle]` commands to `nightctl`.
  * [x] Added `IpcCommand::FocusOutput`, `IpcCommand::MoveWindowToOutput`, and `IpcCommand::Dpms` to IPC protocol.
* [x] **Hotplug Signal Hooks (`crates/night-pinnacle`)**:
  * [x] Attached `OutputSignal::Connect`, `OutputSignal::Disconnect`, and `OutputSignal::Focused` for instant real-time status bar synchronization.

---

### Phase 12: Advanced Window States (Sticky, PiP, Window Centering) & Snowcap Session Flyout
* [x] **Sticky Window Management (`crates/night-binds`)**:
  * [x] Pinned workspace mode (`Super + s`): toggles window visibility across all workspace tags on active monitor.
  * [x] Clean detachment on toggle: unpins back to current active workspace tag only.
* [x] **Picture-in-Picture (PiP) Mode (`crates/night-binds`)**:
  * [x] Picture-in-Picture toggle (`Super + p`): automatically floats, pins sticky, resizes to 16:9 (480x270), and anchors to bottom-right corner.
  * [x] Toggle again restores window to normal tiled mode on active tag.
* [x] **Floating Window Centering (`crates/night-binds`)**:
  * [x] Instant window centering (`Super + c`): calculates display geometry offsets and centers focused floating window.
* [x] **Snowcap Tokyo Night Session Flyout Modal (`crates/night-binds`)**:
  * [x] Interactive in-compositor session manager (`Super + BackSpace` or `nightctl session-menu`).
  * [x] Layer-shell modal with direct keyboard triggers: `[L]` Lock, `[S]` Sleep, `[R]` Reload, `[Q]` Exit, `[P]` Power Off, `[B]` Reboot, `[Escape]` Cancel.
* [x] **CLI & IPC Window Control (`nightctl window`)**:
  * [x] Added `nightctl window <sticky|pip|center|float|fullscreen|maximize>` commands.
  * [x] Added `IpcCommand::WindowOp(WindowAction)` and `IpcCommand::ShowSessionMenu` to IPC protocol.

---

### Phase 13: Slint Control Center & Settings GUI (`crates/night-settings`)
* [x] **Pure-Rust GUI in Slint**:
  * [x] Created `crates/night-settings` using Slint 1.18 GUI toolkit with native Wayland rendering.
  * [x] Implemented Tokyo Night dark palette UI (`#1a1b26`, `#24283b`, `#414868`, `#7aa2f7`, `#c0caf5`).
* [x] **Appearance & Real-time Gaps Tuning**:
  * [x] Real-time inner/outer gaps sliders with instant live compositor preview via IPC (`IpcCommand::SetGaps`).
  * [x] Window border width tuning and configuration saving to `~/.config/night/night.toml`.
  * [x] Layout indicator and click-to-cycle button.
* [x] **Application Preferences**:
  * [x] Visual configuration editor for terminals (`alacritty`, `rio`), browsers (`zen-browser`, `brave`), file managers (`favnyr`, `yazi`), and launchers (`walker`).
* [x] **Displays & Monitor Profiles**:
  * [x] Monitor profile editor reading and writing declarative `~/.config/night/monitors.ron` via `MonitorProfile::save_all()`.
  * [x] HiDPI display scaling presets (`1.0x`, `1.25x`, `1.5x`, `2.0x`), resolution modes, orientation transforms, and Adaptive Sync / VRR toggles.
* [x] **System, Power & Diagnostics**:
  * [x] DPMS display toggle, screen locking (`veila`), compositor reload, system reboot, and power off.
  * [x] Real-time IPC connectivity badge (<2ms latency).
* [x] **Keybinding & Remote Execution**:
  * [x] Bound `Super + i` in `crates/night-binds` to open `night-settings`.
  * [x] Added `nightctl settings` command.
  * [x] Updated `xtask` to build and install `night-settings` to `~/.cargo/bin`.

---

### Phase 14: Visual Window Switcher, Late-Night Ergonomics (Night Light) & Idle Inhibition
* [x] **Snowcap Tokyo Night Visual Window Switcher (`crates/night-binds/src/switcher.rs`)**:
  * [x] Layer-shell interactive window picker modal displaying all open windows across workspace tags and monitor outputs.
  * [x] Displays application badges (`[Alacritty]`, `[Zen]`, `[Favnyr]`), titles, tag identifiers, and display connectors.
  * [x] Direct single-key jump (`[1]..[9]`), `[Enter]` focus & raise, `[D]` close window, `[Esc]` cancel.
  * [x] Automatic cross-output tag activation and window focus.
  * [x] Keybindings: `Super + w` and `Alt + Tab`.
* [x] **Late-Night Ergonomics: Pure-Rust Gamma & Night Light Control (`crates/night-core/src/gamma.rs`)**:
  * [x] Screen color temperature adjustments (2500K–6500K) via Wayland protocols (`wlsunset`, `gammastep`, `hyprsunset`).
  * [x] TOML configuration schema under `[gamma]` in `config/night.toml`.
  * [x] Keybinding: `Super + Shift + g` toggles between neutral daylight (6500K) and warm amber (3800K).
  * [x] `nightctl gamma <toggle|set <temp>|day|night|status>` commands.
  * [x] Slint GUI integration: Live temperature slider and quick presets in `night-settings`.
* [x] **Presentation & Compile Mode: Idle Inhibition**:
  * [x] Inhibit power-saving idle watcher (`wayidle`) to prevent screen lock or DPMS sleep during long compilations, presentations, or video playback.
  * [x] Keybinding: `Super + Shift + p` toggles idle inhibition.
  * [x] `nightctl inhibit` command and real-time status reflection in `DesktopStatus`.
  * [x] Slint GUI toggle button with active state badge.
* [x] **Session Menu & Cheatsheet Synchronization**:
  * [x] Added `[W]` Window Switcher, `[G]` Night Light, and `[I]` Inhibit Mode shortcuts into the `Super + BackSpace` Snowcap Session Flyout.
  * [x] Added `wlsunset` to `cargo xtask doctor`.

---

### Phase 15: Pure-Rust Wayland Clipboard (`rsclip`), Named Multi-Scratchpad Drawers & Modern Media Player (`vivi`)
* [x] **Pure-Rust Wayland Clipboard History Manager (`crates/night-core/src/clipboard.rs` & `crates/night-binds/src/clipboard_ui.rs`)**:
  * [x] Adopted 100% Rust-native `rsclip` (`jR4dh3y/rsclip`) replacing legacy C tools.
  * [x] Daemon management: `rsclipd watch` supervised by `night-daemons` (with fallback to `wl-paste` if `rsclipd` not present).
  * [x] Ephemeral ring buffer caching copied text snippets with `postcard` in `$XDG_RUNTIME_DIR/night-clipboard.bin`.
  * [x] `copy_to_system_clipboard` uses pure-Rust `rsclipd store --mime text/plain`.
  * [x] Snowcap Tokyo Night interactive layer-shell clipboard picker modal (`Super + v` or `nightctl clip show`).
  * [x] Direct launcher for `rsclip` resident GTK4 overlay (`Super + Shift + v`).
  * [x] Direct single-key selection `[1..9]` copies entry back to active system clipboard.
  * [x] `[C]` clears stored clipboard history; `[Esc]` cancels.
* [x] **Modern Pure-Rust Media Player (`vivi` by `Loafer19/vivi`)**:
  * [x] Adopted `vivi` (Rust + Slint UI + `libmpv`) as the official/default media player in `night.toml` (`[apps] media_player = "vivi"`).
  * [x] Bound to `Super + Shift + m` in `night-binds` and designated as the default music scratchpad drawer (`Super + m`).
  * [x] Integrated into Slint Settings GUI Control Center (`night-settings`) in Tab 1 (Applications).
  * [x] Window auto-floating and tag assignment rules configured in `config/rules.ron`.
* [x] **Named Multi-Scratchpad Drawers (`crates/night-binds/src/lib.rs`)**:
  * [x] Generalized scratchpad architecture supporting dedicated workflow drawers:
    * `term`: Full-width dropdown terminal (`Super + ` ` `) anchored to monitor top.
    * `music`: Center floating `vivi` media player (`Super + m`).
    * `notes`: Center floating quick notes editor (`Super + n`).
    * `fm`: Center floating Yazi terminal file manager.
  * [x] Declarative window auto-floating rules in `config/rules.ron`.
  * [x] IPC & CLI control: `nightctl scratchpad [term|music|notes|fm]`.
* [x] **MPRIS Media Player Remote Control**:
  * [x] Hardware multimedia keys mapped in `night-binds`: `XF86AudioPlay`, `XF86AudioPause`, `XF86AudioNext`, `XF86AudioPrev`, `XF86AudioStop`.
  * [x] CLI control: `nightctl media <play-pause|next|prev|stop|status>`.
  * [x] Live media metadata (`{{artist}} - {{title}}`) tracked in `DesktopStatus::media_info` for status bar streaming and `night-settings` integration.

---

### Phase 16: Interactive Tokyo Night Snipping Modal, Workspace Tag Glyphs & 4-Variant Theme Engine
* [x] **Interactive Tokyo Night Snipping & Capture Modal (`crates/night-binds/src/snipping.rs`)**:
  * [x] Snowcap layer-shell modal (`KeyboardInteractivity::Exclusive`, `ZLayer::Overlay`) with instant responsive shortcuts:
    * `[R]`: Snip Region & Annotate (`wayshot -s "$(slurp)" --stdout | satty --filename -`).
    * `[W]`: Snip Focused Window (auto-calculates geometry from `win.loc()` and `win.size()`).
    * `[F]`: Full Screen to File (`~/Pictures/Screenshots/Screenshot_...png`).
    * `[C]`: Full Screen directly to System Clipboard via `rsclipd`.
    * `[V]`: Hardware-accelerated 60fps MP4 Screen Video (`wl-screenrec`).
    * `[G]`: Animated Region Recording as GIF (`wl-screenrec -g "$(slurp)" --output-format gif`).
    * `[Esc] / [Q]`: Close & dismiss modal immediately.
  * [x] Keybound to `Super + Shift + s` in `night-binds`.
  * [x] CLI entrypoint: `nightctl snip [region|window|full|copy|record|gif]`.
* [x] **Rich Workspace Tag Glyphs & Labels (`crates/night-tags/src/lib.rs`)**:
  * [x] Standard Nerd Font glyphs: `󰣆` (Code), `󰈹` (Web), `󰉋` (Files), `󰎆` (Media), `󰙯` (Chat), `󰌨` (Dev), `󰈙` (Docs), `󰭹` (Comm), `󰝰` (Misc).
  * [x] Lookup functions `tag_glyph(name)` and `tag_label(name)`.
  * [x] Integrated into `DesktopStatus::active_tag_glyph`, JSON serialization, and template placeholders `{tag_glyph}` / `{glyph}`.
* [x] **Official 4-Variant Tokyo Night Theme Engine (`crates/night-core/src/theme.rs`)**:
  * [x] Full support for all 4 official variants: `Night` (Classic), `Storm` (Slate), `Moon` (Indigo), and `Day` (Light).
  * [x] Palette specifications: darkest/primary/secondary/highlight backgrounds, primary/muted/dark text, and standard accent colors.
  * [x] Configurable in `night.toml` under `[general] theme = "Night"`.
  * [x] Real-time IPC commands `SetTheme` / `GetTheme` and live status broadcasting.
  * [x] CLI controls: `nightctl theme <set <night|storm|moon|day>|toggle|status>`.
  * [x] `cargo xtask theme [night|storm|moon|day]` generating variant-tailored `alacritty.toml` and `ironbar.css`.
* [x] **Slint Settings Control Center Theme & Snipping Integration (`crates/night-settings`)**:
  * [x] Interactive theme variant selector buttons (`Night`, `Storm`, `Moon`, `Day`) in Tab 0 (Appearance) with live IPC updating and persistent disk saving.
  * [x] Quick launch button for Tokyo Night Snipping Modal in Tab 3 (System & Session).

---

### Phase 17: Pure-Rust Battery & Power Monitor, Audio Controls, Dynamic Wallpaper-Theme Engine & Auto-Reloading
* [x] **Pure-Rust Battery & Power Supply Monitor (`crates/night-core/src/power.rs` & `crates/night-daemons/src/power_watcher.rs`)**:
  * [x] Zero-dependency sysfs reader for Linux `/sys/class/power_supply` (`BAT0`, `BAT1`, `ADP1`).
  * [x] Dynamic Nerd Font icons (`󰂄` charging, `󰁹`, `󰂂`, `󰂀`, `󰁾`, `󰁼`, `󰁺`).
  * [x] Background supervisor emitting desktop notification alerts at low (<=15%) and critical (<=5%) battery levels via `notify-send` / `wired`.
  * [x] Integrated into `DesktopStatus::battery`, JSON output, and template placeholders `{battery}`, `{battery_pct}`, `{battery_icon}`.
  * [x] `nightctl power` command to query battery state.
* [x] **Audio Volume Management & SwayOSD Synchronization (`crates/night-core/src/audio.rs`)**:
  * [x] Direct volume control via PipeWire (`wpctl`) and PulseAudio (`pactl`).
  * [x] `VolumeAction`: `Raise(delta)`, `Lower(delta)`, `Set(percentage)`, `MuteToggle`, `Query`.
  * [x] Automatic on-screen display (OSD) feedback via `swayosd-client`.
  * [x] Hardware multimedia keys (`XF86AudioRaiseVolume`, `XF86AudioLowerVolume`, `XF86AudioMute`) mapped in `night-binds` with real-time IPC broadcast.
  * [x] Integrated into `DesktopStatus::volume`, template placeholders `{volume}`, `{volume_pct}`, `{volume_icon}`.
  * [x] `nightctl volume <raise|lower|set|mute|status>` CLI suite.
* [x] **Dynamic Wallpaper-Theme Engine (`crates/night-daemons/src/wallpaper.rs`)**:
  * [x] Synchronizes active Tokyo Night theme variant (`Night`, `Storm`, `Moon`, `Day`) with `swww` wallpaper transitions.
  * [x] Smooth wipe transitions (`--transition-type wipe --transition-duration 1.5 --transition-angle 30`).
  * [x] `nightctl wallpaper [path]` to inspect or apply desktop wallpapers.
* [x] **Live Configuration File Auto-Reloading (`crates/night-daemons/src/config_watcher.rs`)**:
  * [x] Background watcher polling `rules.ron`, `monitors.ron`, and `night.toml` file timestamps.
  * [x] Dynamically hot-reloads window placement rules, display profiles, and theme preferences without restarting Pinnacle.
* [x] **Slint Control Center System Integration (`crates/night-settings`)**:
  * [x] Tab 3 (System & Power): Added live battery status badge and interactive master volume slider.

---

### Phase 18: Pure-Rust Snowcap Tokyo Night Fuzzy App Launcher, Network Connectivity Monitor & System Synchronization
* [x] **Pure-Rust Snowcap Application Launcher & Runner Modal (`crates/night-binds/src/launcher.rs`)**:
  * [x] Fast, in-process XDG desktop application scanner inspecting `/usr/share/applications`, `~/.local/share/applications`, and `/usr/local/share/applications`.
  * [x] Cleans XDG exec arguments (`%f`, `%u`, `%F`, etc.) and automatically wraps terminal applications.
  * [x] Curated priority ordering for signature Tokyo Night tools (`Alacritty`, `Rio`, `Vivi`, `Favnyr`, `Yazi`, `Zen Browser`, `Brave`, `Night Settings`).
  * [x] Snowcap layer-shell modal bound to `Super + space` (and `nightctl launcher`).
  * [x] Interactive single-key instant launch (`[1]..=[8]`), extended search (`[9]` Walker), mnemonic shortcuts (`[T]`, `[M]`, `[F]`, `[B]`, `[S]`), and `[Esc]` cancellation.
* [x] **Pure-Rust Network Connectivity & Wi-Fi Monitor (`crates/night-core/src/network.rs`)**:
  * [x] Direct sysfs `/sys/class/net` operstate scanner for active wired (`en*`, `eth*`) and wireless (`wl*`) interfaces.
  * [x] Dynamic Nerd Font signal quality glyphs: `󰤨` (80-100%), `󰤥` (60-79%), `󰤢` (40-59%), `󰤟` (20-39%), `󰤯` (<20%), `󰈀` (Ethernet), `󰤮` (Disconnected).
  * [x] Active SSID, signal strength, and local IPv4 address resolution.
  * [x] Integrated into `DesktopStatus::network`, JSON serialization, and status bar template placeholders `{network}`, `{net_icon}`, `{ssid}`, `{ip}`.
  * [x] CLI inspection command: `nightctl net`.
* [x] **Slint Control Center Network Integration (`crates/night-settings`)**:
  * [x] Tab 3 (System & Power): Added live Network Connectivity indicator row synchronized with live IPC status.
* [x] **Cheatsheet & Session Menu Synchronization**:
  * [x] `SessionFlyout`: Added visual display of `[V] Clipboard (rsclip)` and `[Space] App Launcher` in the 4th action row.
  * [x] `xtask doctor`: Hardened 100% Rust-native diagnostics for `rsclip` and `vivi` with zero C-native tooling dependencies.
  * [x] Mapped hardware media keys and IPC media actions to query active MPRIS players (including Vivi) via native D-Bus / `busctl`.
  * [x] Updated snipping modal descriptions and screenshot scripts to exclusively use `rsclip` (`rsclipd`).

---

### Phase 19: Pure-Rust System Performance Monitor, Bluetooth Peripheral Manager & Interactive Audio Sink Switcher Modal
* [x] **Zero-Dependency Linux Procfs Performance Monitor (`crates/night-core/src/sysinfo.rs`)**:
  * [x] Real-time CPU calculation from `/proc/stat` delta sampling without external dependencies.
  * [x] Memory usage (total, used, percentage) parsed directly from `/proc/meminfo`.
  * [x] CPU temperature monitoring via `/sys/class/thermal/thermal_zone0/temp` and hwmon.
  * [x] Root filesystem `/` disk metrics via standard filesystem stats.
  * [x] Dynamic Nerd Font glyphs: `` (CPU), `󰍛` (Memory), `󰔏` (Thermal), `󰋊` (Disk).
  * [x] Integrated into `DesktopStatus::system`, JSON serialization, and status bar template placeholders `{cpu}`, `{cpu_pct}`, `{mem}`, `{mem_pct}`, `{temp}`, `{disk}`.
  * [x] `nightctl top` / `nightctl sys`: High-performance terminal dashboard with ASCII gauges for CPU, RAM, Disk, and Temperature.
* [x] **Pure-Rust Bluetooth Peripheral & Controller Monitor (`crates/night-core/src/bluetooth.rs`)**:
  * [x] Zero-dependency BlueZ manager inspecting `/sys/class/bluetooth` adapter state and RFKill soft blocks.
  * [x] Connected wireless peripheral discovery (headsets, mice, keyboards, game controllers).
  * [x] Automatic battery reporting via `/sys/class/power_supply` for paired Bluetooth devices.
  * [x] Integrated into `DesktopStatus::bluetooth`, JSON output, and template placeholders `{bt}`, `{bt_icon}`.
  * [x] `nightctl bt` CLI command to inspect Bluetooth state and connected accessories.
* [x] **Interactive Audio Output & Sink Switcher (`crates/night-core/src/audio.rs` & `crates/night-binds/src/audio_ui.rs`)**:
  * [x] PipeWire (`wpctl status`) and PulseAudio (`pactl list sinks short`) device parser discovering active audio sinks.
  * [x] Single-key Snowcap layer-shell modal (`show_audio_sink_modal`) bound to `Super + Shift + a`.
  * [x] Integrated into Session Flyout (`Super + BackSpace` -> `[A] Audio Sinks`).
  * [x] `nightctl sink <list|cycle|set <id>|menu>` CLI management suite.
* [x] **Slint Settings Control Center Hardware & Peripheral Integration (`crates/night-settings`)**:
  * [x] Tab 3 (System & Power): Added live Bluetooth wireless peripheral status badge.
  * [x] Tab 3 (System & Power): Added live CPU utilization and Memory (RAM) Tokyo Night progress meters.

---

### Phase 20: Display Backlight Engine, Do-Not-Disturb (DND) Center, Notification History Modal & Session Restore Engine
* [x] **Pure-Rust Display Backlight Control Engine (`crates/night-core/src/backlight.rs`)**:
  * [x] Zero-dependency `/sys/class/backlight` reader discovering hardware brightness devices (`intel_backlight`, `amdgpu_bl0`, etc.).
  * [x] Seamless adjustment via `brightnessctl` and `swayosd-client` on-screen feedback.
  * [x] Dynamic Nerd Font glyphs: `󰃠` (75-100%), `󰃟` (50-74%), `󰃞` (25-49%), `󰃝` (0-24%).
  * [x] Hardware brightness keys (`XF86MonBrightnessUp`, `XF86MonBrightnessDown`) mapped to direct backlight control in `night-binds`.
  * [x] Integrated into `DesktopStatus::backlight`, JSON output, and template placeholders `{brightness}`, `{brightness_pct}`, `{brightness_icon}`.
  * [x] `nightctl brightness [raise|lower|set <pct>|status]` CLI suite.
* [x] **Notification Center & Do Not Disturb (DND) Daemon (`crates/night-core/src/notifications.rs` & `crates/night-binds/src/notifications_ui.rs`)**:
  * [x] Postcard binary ring-buffer caching up to 30 notification records in `$XDG_RUNTIME_DIR/night-notifications.bin`.
  * [x] Do Not Disturb (DND) state toggle suppressing and dropping desktop notifications via `wired --dnd`.
  * [x] Snowcap layer-shell Notification Center Modal (`Super + Shift + n`) showing recent notifications, timestamps, and app tags.
  * [x] Single-key actions: `[D]` Toggle DND, `[C]` Clear History, `[Esc]` Cancel.
  * [x] Session flyout integration: Added `[N] Notifications` to `Super + BackSpace`.
  * [x] Integrated into `DesktopStatus::dnd`, template placeholders `{dnd}`, `{dnd_icon}` (`󰂛` / `󰂚`).
  * [x] `nightctl dnd [toggle|on|off|status]` and `nightctl notify [list|clear|menu]`.
* [x] **Session State Persistence & Workspace Window Restoration (`crates/night-core/src/session_restore.rs`)**:
  * [x] Serializes running window layout, app IDs, titles, assigned tags, and floating states into `$XDG_RUNTIME_DIR/night-saved-session.bin` via `postcard`.
  * [x] `nightctl session <save|restore|clear>` CLI controls.
* [x] **Slint Settings Control Center Backlight & DND Integration (`crates/night-settings`)**:
  * [x] Tab 3 (System & Power): Added interactive Display Backlight slider with real-time percentage feedback.
  * [x] Tab 3 (System & Power): Added interactive Do Not Disturb (DND) toggle button with visual state changes.

---

### Phase 21: Production-Grade Non-Blocking Logging Architecture, Application Launch Resiliency, Visual OSD Engine & Dynamic Workspace Layout Profiles
* [x] **Production-Grade Non-Blocking Logging Architecture (`crates/night-core/src/logging.rs`)**:
  * [x] Integrated `tracing-appender = "0.2"` daily rolling file logger writing uncolored, timestamped, thread-indexed logs to `~/.local/state/night/logs/<service>.log`.
  * [x] Automated fallback to `/tmp/night/logs/` if state dir permissions or sandboxes restrict user directory creation.
  * [x] Global panic hook installation via `std::panic::set_hook` capturing panic payload, source location, thread ID, and full backtrace directly to the log file before process termination.
  * [x] Integrated across all services: `night` (`night-pinnacle`), `settings` (`night-settings`), `ctl` (`nightctl`), and `session` (`night-session`).
  * [x] Real-time log inspection CLI commands: `nightctl logs [service] [-n lines] [-f]` and `cargo xtask logs [service]`.
* [x] **Desktop Application Launch Hardening & Reliability Repair (`crates/night-core/src/exec.rs`)**:
  * [x] `ensure_desktop_env()`: Automatically prepends `~/.cargo/bin`, `~/.local/bin`, and `/usr/local/bin` to `PATH` upon startup to prevent missing binary errors when launched under systemd/greetd.
  * [x] Dynamic `WAYLAND_DISPLAY` auto-detection scanning `$XDG_RUNTIME_DIR` for active sockets (`wayland-0`, `wayland-1`).
  * [x] Added `resolve_executable()` thoroughly probing standard PATH, Cargo binaries, target release/debug artifacts, and system paths.
  * [x] Resilient, decoupled child process execution: safely attempts `pinnacle_api` process spawning when inside compositor runtime, with seamless decoupled `std::process::Command` fallback (`Stdio::null()`) to prevent crashes when called from standalone tools or Slint GUI threads.
  * [x] Updated Snowcap application launcher modal (`crates/night-binds/src/launcher.rs`) to resolve executables via `resolve_executable()`.
* [x] **Slint Settings Control Center Resiliency (`crates/night-settings`)**:
  * [x] Guaranteed non-crashing initialization with `init_logger("settings")` and `ensure_desktop_env()`.
  * [x] Fixed panic in poweroff/reboot buttons by decoupling process spawning from `pinnacle_api`.
  * [x] Added declarative floating window rule `Rule(app_id: "night-settings", action: Float)` in `config/rules.ron` and `crates/night-rules/src/lib.rs`.
* [x] **Pure-Rust Visual On-Screen Display (OSD) Engine (`crates/night-binds/src/osd.rs`)**:
  * [x] In-process Snowcap layer-shell visual HUD overlay replacing external C tools.
  * [x] Displays Tokyo Night visual progress bars and Nerd Font icons for Volume, Brightness, DND, Gamma, and Layout switching.
  * [x] Non-blocking auto-dismissal after 1200ms duration.
  * [x] Hooked directly into hardware multimedia keys (`XF86Audio*`, `XF86MonBrightness*`), layout cycling (`Super + Tab`), and IPC triggers (`nightctl osd <target>`).
* [x] **Dynamic Tiling Workspace Layout Profiles Engine (`crates/night-layout/src/profiles.rs`)**:
  * [x] 4 ergonomic workflow presets: `Coding` (MasterStack 60/40, 8px/12px), `Reading` (MasterStack 32px margins), `Multitasking` (Fair 4-quadrant grid), `Ultrawide` (Spiral borderless edge).
  * [x] Dedicated keybindings: `Super + Alt + 1` (Coding), `Super + Alt + 2` (Reading), `Super + Alt + 3` (Multitasking), `Super + Alt + 4` (Ultrawide).
  * [x] Integrated into IPC commands (`IpcCommand::SetLayoutProfile`), status broadcasting, and CLI `nightctl profile [name]`.
  * [x] Slint Settings Control Center Integration: Added 4 interactive Layout Profile preset cards to Tab 0 (Appearance & Layout).

---

### Phase 22: Pure-Rust Session Management Hardening & Systemd-Logind Sleep-Lock Daemon
* [x] **Systemd-Logind Sleep & Power Signal Daemon (`crates/night-daemons/src/sleep_watcher.rs`)**:
  * [x] Pure D-Bus / systemd monitor tracking `org.freedesktop.login1.Manager.PrepareForSleep` signal.
  * [x] Automatically engages session screen lock (`night-session lock` / `veila`) and blanks displays via DPMS before entering sleep.
  * [x] Seamlessly restores display power and wakes outputs when system resumes from sleep/suspend.
* [x] **Modernized Pure-Rust Session Management CLI & Binary (`crates/night-session`)**:
  * [x] Upgraded `night-session` with `clap` subcommands: `run`, `lock`, `logout`, and `status`.
  * [x] High-performance graceful graphical teardown handling POSIX signals (`SIGINT`, `SIGTERM`).
  * [x] Direct CLI integration: `nightctl session <lock|logout|status|save|restore>`.
* [x] **Slint Settings Control Center Power Integration (`crates/night-settings`)**:
  * [x] Tab 3 (System & Power): Added interactive `Lock Session`, `Sleep / Suspend`, and `Logout Desktop` action buttons.
* [x] **Ergonomic Session Keybindings (`crates/night-binds`)**:
  * [x] `Super + Alt + L`: Lock active desktop session.
  * [x] `Super + Alt + S`: Suspend system to RAM.
  * [x] `Super + Shift + Escape`: Graceful desktop logout.

---

### Phase 23: Interactive Window Overview, Floating Snapping & PiP Engine, Desktop Command Palette & Smart Idle Daemon
* [x] **Floating Window Snapping, Picture-in-Picture & Geometry Engine (`crates/night-layout/src/floating.rs`)**:
  * [x] Precise screen geometry calculation supporting 11 targets: `Center`, `LeftHalf`, `RightHalf`, `TopHalf`, `BottomHalf`, `TopLeft`, `TopRight`, `BottomLeft`, `BottomRight`, `Pip`, and `Maximize`.
  * [x] Computes exact monitor bounds respecting inner gaps, outer margins, and layer-shell boundaries.
  * [x] Picture-in-Picture (PiP) mode: docks floating window to bottom-right corner (32% width, 16:9 ratio, 24px padding), raises it, and pins sticky across all workspace tags.
  * [x] Dedicated keybindings: `Super + Shift + C` (Center), `Super + Alt + Left/Right/Up/Down` (Half splits), `Super + Alt + u/i/j/k` (Quadrants), `Super + Alt + P` (PiP).
  * [x] CLI & IPC integration: `nightctl snap <position>` and `IpcCommand::SnapWindow(pos)`.
* [x] **Interactive Tokyo Night Desktop Command Palette (`crates/night-binds/src/palette.rs`)**:
  * [x] Snowcap layer-shell modal (`Super + P` / `Super + Shift + P` or `nightctl palette`).
  * [x] Instant keyboard execution for 12 core desktop operations: App Launcher (`[1]`), Window Switcher (`[2]`), Layout Profiles (`[3]`), Theme Variants (`[4]`), Snipping Studio (`[5]`), Audio Output (`[6]`), DND Mode (`[7]`), Night Light (`[8]`), Control Center (`[9]`), Session Lock (`[0]`), Suspend (`[S]`), Logout (`[X]`).
* [x] **Enhanced Tokyo Night Window Overview & Switcher Modal (`crates/night-binds/src/switcher.rs`)**:
  * [x] Real-time window inspection enumerating open windows across all workspace tags and outputs.
  * [x] Contextual Tokyo Night Nerd Font app icons (`` Terminal, `` Browser, `` Vivi, `󰉋` Files, `` Settings, `󰅩` Code).
  * [x] Dynamic `[Tile]` vs `[Float]` window mode indicators and active workspace tag tags.
  * [x] Single-key instant focus jump (`[1..9]`), `[Enter]` focus, `[X] / [D]` close window, `[Esc]` cancel.
  * [x] Bound to `Super + W` and `Alt + Tab` (and `nightctl overview`).
* [x] **Pure-Rust Smart Idle & Inactivity Supervision Daemon (`crates/night-daemons/src/idle_watcher.rs`)**:
  * [x] Autonomous background supervisor checking user activity and media playback state every 10 seconds.
  * [x] Media/presentation inhibition detection: automatically suppresses idle dimming/locking when video or audio playback is active.
  * [x] Staged power management: Dims backlight to 50% after 300s (5m), locks session via `night-session lock` after 600s (10m), and enters DPMS sleep after 900s (15m).
  * [x] Instant activity resumption: restores original brightness and powers display on upon input.
  * [x] CLI management: `nightctl idle status` and `nightctl idle reset`.
* [x] **Cheatsheet & Session Flyout Synchronization (`crates/night-binds/src/lib.rs`)**:
  * [x] Added `[C] Palette` to `SessionFlyout` (`Super + BackSpace`).
  * [x] Purged all remaining references to legacy C tools (`wl-clipboard`, `playerctl`); 100% Rust-native tooling with `rsclip`, `vivi`, and native D-Bus.

---

### Phase 24: Dynamic Per-Window Titlebars, Window Accent Borders & Multi-Monitor Scratchpad Anchoring
* [x] **Window Focus Border Accents & Active Frame Rendering (`crates/night-binds/src/decorations.rs`)**:
  * [x] Integrated Snowcap `pinnacle_api::snowcap::FocusBorder` for active window frame decorations.
  * [x] Tokyo Night theme-tailored focus frame accents (`#7aa2f7` blue) and unfocused highlight borders (`#414868`).
  * [x] Glowing Tokyo Night magenta accents (`#bb9af7`) on floating and PiP windows.
  * [x] Optional native window titlebars with close and maximize controls.
  * [x] Real-time keyboard toggles: `Super + Alt + b` (Titlebars) and `Super + Alt + d` (Border Decorations).
  * [x] CLI & IPC management: `nightctl border <toggle|titlebars|width <px>>` and `IpcCommand::ToggleDecorations` / `ToggleTitlebars` / `SetBorderWidth`.
* [x] **Multi-Monitor Tag Pinning & Scratchpad Dynamic Output Migration (`crates/night-binds/src/lib.rs`)**:
  * [x] Scratchpads automatically migrate to follow the active monitor output upon toggle.
  * [x] Clears orphan workspace tags across prior screens and recalculates responsive dimensions and positioning for the focused display.

---

### Phase 25: Terminal Window Swallowing Engine, Bihar Night Workspace Grid (Exposé), Live Decoration Controls in Settings GUI & Presentation Projection Engine
* [x] **Terminal Window Swallowing & Re-Parenting Engine (`crates/night-rules/src/swallow.rs`)**:
  * [x] Pure-Rust process tree & window lifecycle monitor temporarily hiding parent terminal windows when child graphical applications are spawned.
  * [x] Automatic restoration on child destruction: listens to `WindowSignal::Destroyed` to automatically unswallow, raise, and refocus the parent terminal.
  * [x] Declarative RON rule integration: added `WindowAction::Swallow` support in `config/rules.ron`.
  * [x] Manual swallowing controls: `nightctl window swallow` and `nightctl window unswallow`.
* [x] **Bihar Night 3x3 Workspace Grid Exposé Modal (`crates/night-binds/src/grid.rs`)**:
  * [x] Interactive Snowcap layer-shell modal rendering a 3x3 grid overview for workspace tags `[1]..[9]`.
  * [x] Displays tag glyphs, status badges (`ACTIVE`, `OCCUPIED`, `EMPTY`), window count indicators, and window title previews.
  * [x] Single-key shortcuts: `[1]..=[9]` instant jump, `[Shift + 1..9]` move focused window to tag, `[Esc/Q]` dismiss.
  * [x] Dedicated keybindings: `Super + g` and `Super + Shift + w`.
  * [x] Integrated into Command Palette (`Super + P` -> `[G]`), Session Flyout (`Super + BackSpace` -> `[M]`), and CLI (`nightctl grid`).
* [x] **Slint Control Center Live Decoration & Titlebar Tuning (`crates/night-settings`)**:
  * [x] Added "Window Focus Borders & Titlebars" card to Tab 0 (Appearance & Layout) in `night-settings`.
  * [x] Interactive toggle buttons for Focus Borders and Native Titlebars with live preview.
  * [x] Live sliders for Border Width (1px – 10px) and Titlebar Height (14px – 32px) communicating in real-time via IPC.
  * [x] Persistent saving directly to `~/.config/night/night.toml`.
* [x] **Multi-Head Presentation & Display Swap Engine (`crates/night-tags` & `nightctl output`)**:
  * [x] `night_tags::swap_active_outputs()`: Swaps active workspaces and client windows between primary and secondary monitors instantly.
  * [x] `night_tags::mirror_active_output_to_next()`: Mirrors active workspace tag and windows onto external display or projector.
  * [x] Dedicated keybinding: `Super + Alt + o` to swap display workspaces.
  * [x] CLI commands: `nightctl output swap` and `nightctl output mirror`.

---

### Phase 26: The Bihar Night Identity, 4-Variant Cultural Engine & High-Resolution Default Wallpapers
* [x] **Bihar Night Cultural Identity & Philosophy Documentation (`BIHAR_NIGHT.md`)**:
  * [x] Formulated foundational aesthetic principles rooted in Bodh Gaya enlightenment starlight, River Ganga dusk Aarti & Mahatma Gandhi Setu night horizons, Aryabhata's celestial mathematics at Taregana / Nalanda, Rajgir misty basalt hills, and Mithila (Madhubani) natural indigo/terracotta pigments.
  * [x] Comprehensive architectural document `BIHAR_NIGHT.md` detailing design philosophies, WCAG AAA contrast ratios, cultural anchors, and color tables.
* [x] **Bihar Night 4-Variant Theme Engine (`crates/night-core/src/theme.rs`)**:
  * [x] Rebranded and enriched the 4 theme expressions:
    * `Night`: **Ganga Midnight** (River Ganga dusk, golden diyas `#e0af68`, obsidian river water `#16161e` / `#1a1b26`).
    * `Storm`: **Rajgir Mist** (Ratnagiri slate basalt hills `#1f2335` / `#24283b`, bamboo grove green `#9ece6a`, moonlit stupa).
    * `Moon`: **Bodh Gaya Serenity** (Mahabodhi tree starlight indigo `#1e2030` / `#222436`, brass temple bell gold `#ffc777`, lotus magenta `#fca7ea`).
    * `Day`: **Nalanda Dawn** (Ancient terracotta sandstone ruins `#e1e2e7`, manuscript carbon ink `#3760bf`, kiln brick red `#f52a65`).
  * [x] Added `variant.bihar_name()` and `variant.inspiration()`.
  * [x] Extended `ThemeVariant::parse_variant()` with cultural aliases: `ganga`, `mithila`, `patna`, `rajgir`, `mist`, `bodhgaya`, `bodhi`, `nalanda`, `dawn`.
* [x] **Curated Authentic High-Resolution Default Wallpapers (`assets/wallpapers/`)**:
  * [x] Packaged `bihar-night-ganga.jpg` (5184x3456 5K UHD): River Ganga & Mahatma Gandhi Setu under the Super Moon Night at Patna.
  * [x] Packaged `bihar-night-bodhgaya.jpg` (4219x3164 4K UHD): Mahabodhi Temple and Garden illuminated at night at Bodh Gaya.
  * [x] Established `assets/wallpapers/bihar-night.jpg` as the primary default desktop wallpaper across all daemons and scripts.
* [x] **Wallpaper Pipeline Synchronization (`assets/wallpaper.sh`, `night-daemons`, `xtask`)**:
  * [x] Updated `assets/wallpaper.sh` to search signature Bihar Night wallpapers first with smooth grow transitions and `#1a1b26` solid fallback.
  * [x] Updated `crates/night-daemons/src/wallpaper.rs` to prioritize `bihar-night*.jpg` in user and asset directories.
  * [x] Updated `cargo xtask wallpaper` and `cargo xtask install` to automatically provision `~/.config/night/wallpapers/` on setup.
* [x] **Desktop Suite & UI String Alignment Across All Crates**:
  * [x] Updated Slint Control Center GUI (`crates/night-settings`): "Bihar Night Theme Engine", "Ganga Midnight", "Rajgir Mist", "Bodh Gaya", "Nalanda Dawn".
  * [x] Updated CLI (`nightctl theme set [ganga|rajgir|bodhgaya|nalanda]`, `nightctl theme status`).
  * [x] Updated all Snowcap layer-shell modals in `night-binds`: Command Palette, Switcher, 3x3 Grid, Snipping, Audio Selector, Notification Center, Session Flyout.
  * [x] Updated `cargo xtask theme` exporter generating Bihar Night headers for Alacritty, Ironbar, Anyrun, and Veila.

---

### Phase 27: Diurnal Theme Scheduler, Bihar Night Battery Conservation Mode, and Monocle Fullscreen-Focus Layout Engine
* [x] **Monocle Fullscreen-Focus Layout Engine (`crates/night-layout` & `crates/night-binds`)**:
  * [x] Added `LayoutKind::Monocle`: Distraction-free single-window active workspace layout with 0px inner/outer gaps and 100% (1.0) master factor.
  * [x] Added `LayoutKind::parse()` supporting "monocle", "fullscreen", "max", "focus", "masterstack", "dwindle", "spiral", "fair".
  * [x] Extended `LayoutProfile::Focus` with unit tests passing.
  * [x] Bound `Super + Alt + 5` to `LayoutProfile::Focus` in `night-binds`.
  * [x] Added `nightctl layout set <name>` and direct subcommands (`nightctl layout monocle`, `masterstack`, `dwindle`, `spiral`, `fair`).
* [x] **Diurnal Solar Theme Scheduler (`crates/night-daemons`, `night-core`, `nightctl`, `night-settings`)**:
  * [x] Created `diurnal_watcher.rs` supervising time-of-day solar phases:
    * 06:00 – 18:00: **Nalanda Dawn** (Neutral daylight, 6500K color temperature).
    * 18:00 – 22:00: **Rajgir Mist** (Warm evening twilight, 4500K color temperature).
    * 22:00 – 06:00: **Ganga Midnight** (OLED deep obsidian night, 3400K color temperature).
  * [x] Added `auto_theme_schedule` flag to `GeneralConfig` in `night-core`.
  * [x] Added IPC protocol commands `ToggleAutoTheme`, `SetAutoTheme(bool)`, `GetAutoTheme` and responses.
  * [x] Extended `nightctl theme schedule [on|off|toggle|status]` with clear terminal feedback.
  * [x] Integrated interactive Diurnal Solar Schedule toggle button and schedule description in Slint Control Center (Appearance tab).
* [x] **Bihar Night Battery Conservation Mode (`crates/night-daemons`, `night-core`, `nightctl`, `night-settings`)**:
  * [x] Upgraded `power_watcher.rs` with `is_power_saver()` and `set_power_saver(bool)`.
  * [x] Automatic energy conservation triggers: dims display backlight to 30%, switches active theme to OLED Ganga Midnight, and emits notification banners. Auto-restores normal 65% brightness upon AC adapter reconnection.
  * [x] Added `power_saver_mode` flag to `GeneralConfig` in `night-core`.
  * [x] Added IPC protocol commands `TogglePowerSaver`, `SetPowerSaver(bool)`, `GetPowerSaver` and responses.
  * [x] Extended `nightctl power saver [on|off|toggle|status]` with battery status feedback.
  * [x] Integrated interactive Battery Conservation Mode toggle button and description in Slint Control Center (System & Power tab).
* [x] **Compositor & Multi-Client IPC Broadcast Synchronization**:
  * [x] Updated `crates/night-pinnacle/src/main.rs` handling `SetLayout`, `ToggleAutoTheme`, `SetAutoTheme`, `GetAutoTheme`, `TogglePowerSaver`, `SetPowerSaver`, `GetPowerSaver`.
  * [x] Populated `DesktopStatus::to_json()` and `DesktopStatus::format_template()` with `auto_theme` and `power_saver`.
  * [x] Enforced real-time IPC broadcast notifications across running status bars and control centers.

---

### Phase 28: Walker + Elephant Launcher Integration, Daemon Health Supervisor & `nightctl doctor`
* [x] **Walker + Elephant Integration (`night-core`, `night-daemons`, `night-binds`, `xtask`)**:
  * [x] Replaced legacy launcher defaults with `walker` + `elephant` in `night.toml`, `Defaults`, `AppsConfig`, `settings.slint`, and `packaging/PKGBUILD`.
  * [x] Added dynamic `ThemeVariant::sync_walker_theme()` generating `~/.config/walker/themes/night/style.css` on startup, theme switch, and `night.toml` hot-reload.
  * [x] Bound `Super + Space` / `Alt + Space` to `walker` (with Snowcap fallback), `Super + .` to Walker symbols/emoji picker, and `Super + =` to Walker calculator.
* [x] **Daemon Health & Auto-Restart Supervisor (`crates/night-daemons` & `night-core`)**:
  * [x] Added `/proc` duplicate-spawn guard and `supervise_core_daemons()` auto-restarting crashed core daemons (up to 5 restarts).
  * [x] Added `IpcCommand::GetDaemons`, `IpcResponse::Daemons(Vec<DaemonHealth>)`, and `nightctl daemons`.
* [x] **Standalone Diagnostics & Launcher CLI (`crates/night-ctl`)**:
  * [x] Extended `nightctl launcher [apps|calc|files|clipboard|symbols|websearch|runner|dmenu|snowcap]`.
  * [x] Added `nightctl doctor` checking `WAYLAND_DISPLAY`, Night IPC connectivity, and core desktop binaries.

---

### Phase 29: Persistent Session Recovery, `app_id` Launch Resolution & `interprocess` IPC
* [x] **`interprocess` + `LengthDelimitedCodec` IPC Migration (`crates/night-core/src/ipc.rs`)**:
  * [x] Replaced raw `tokio::net::UnixStream`/`UnixListener` and manual byte-length framing with `interprocess::local_socket::tokio` and `tokio_util::codec::LengthDelimitedCodec` (4 MiB frame cap).
  * [x] Added `test_interprocess_ipc_roundtrip` verifying live client/server IPC roundtrips.
* [x] **Persistent Session Recovery (`crates/night-core/src/session_restore.rs`, `night-pinnacle`, `night-ctl`)**:
  * [x] Moved `SavedSession::cache_path()` from volatile `$XDG_RUNTIME_DIR` to persistent `$XDG_DATA_HOME/night/saved-session.bin` so sessions survive logouts and reboots.
  * [x] Added `resolve_launch_command()` mapping Wayland `app_id`s (reverse-DNS/branded IDs) to system executables.
  * [x] Updated `SaveSession` / `RestoreSession` to capture per-window tags and active layout, and restore theme, layout, active tag, and windows.
  * [x] Enhanced `nightctl session status` to inspect saved session snapshots on disk.












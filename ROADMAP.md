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
Phase 1 ──> Phase 2 ──> Phase 3 ──> Phase 4 ──> Phase 5 ──> Phase 6 ──> Phase 7 ──> Phase 8 ──> Phase 9 ──> Phase 10 ──> Phase 11 ──> Phase 12
[Workspace] [Formats/Task] [Interactivity] [xtask Flow] [Hardening] [Layout/Theme] [Signals/Doctor] [Power Features] [Bar/Snowcap/nightctl] [Packaging/Media] [Multi-Monitor/DPMS] [Window Ops/Session Flyout]
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




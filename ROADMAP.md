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
Phase 1 ──> Phase 2 ──> Phase 3 ──> Phase 4 ──> Phase 5
[Workspace] [Formats/Task] [Interactivity] [xtask Flow] [Hardening]
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

### Phase 5: Multi-Monitor, Theming & Hardening
* [x] Unified Tokyo Night dark color palette in `night-core::theme`.
* [ ] Multi-monitor specific tag layout handling and screen resolution profiles.
* [ ] Verify wayidle session lock integration with `swaylock`.

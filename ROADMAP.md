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

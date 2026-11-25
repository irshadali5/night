# 🛠️ Project Night: Implementation Tracker

This document tracks the technical specifications, format design choices, and implementation status for the **Night** desktop environment.

---

## 💎 The Four Pillars: Architecture & Format Selection

Each technology has been chosen and implemented for its ideal use case:

| Technology | Implementation & Target | Purpose & Why It Fits Best |
| :--- | :--- | :--- |
| 🦀 **`cargo xtask`** | [`xtask/`](./xtask), `.cargo/config.toml` | **Workspace Automation:** Pure Rust CLI task runner replacing Makefiles and shell scripts (`cargo xtask install`, `check`, `build`, `run`, `uninstall`). |
| 📄 **TOML** | [`config/night.toml`](./config/night.toml), `night-core::config` | **Human-Editable Preferences:** Clean, readable key-value format for top-level desktop settings, default terminals, launchers, and borders. |
| 🦀 **RON** | [`config/rules.ron`](./config/rules.ron), `night-rules` | **Rich Rust Types & Enums:** Strongly-typed representation for complex window matching rules and actions (`Float`, `MoveToTag("8")`) without string hacks. |
| ⚡ **`postcard`** | `night-core::state` $\rightarrow$ `$XDG_RUNTIME_DIR/night-session.bin` | **Ultra-Fast Binary Session Caching:** Zero-allocation binary serde format for instant state preservation/restoration across reloads. |

---

## 🏗️ Workspace Crates Matrix

| Crate | Type | Dependencies / Format | Role | Status |
| :--- | :--- | :--- | :--- | :--- |
| **`xtask`** | Binary | `night-core`, `tokio`, `dirs`, `anyhow` | Native CLI runner for build, install, check, doctor, logs, and IPC control (`cargo xtask`). | 🟢 Active |
| **`night-core`** | Library | `dirs`, `interprocess`, `which`, `thiserror`, `sysinfo`, `notify-rust`, `postcard`, `toml`, `serde` | Themes, default apps, typed errors, sysinfo metrics, D-Bus notifications, and IPC socket. | 🟢 Active |
| **`night-daemons`** | Library | `night-core`, `which`, `pinnacle-api` | Supervises background daemons (`rsclip`, `vivi`, `swww`, `ironbar`, `wired`, `swayosd`, `wayidle`, idle/sleep watchers). | 🟢 Active |
| **`night-tags`** | Library | `night-core`, `pinnacle-api` | Multi-monitor dynamic tags ([1..9]), display hotplug, workspace swapping, and output projection. | 🟢 Active |
| **`night-layout`**| Library | `night-core`, `pinnacle-api` | Dynamic tiling layout engine (MasterStack, Dwindle, Spiral, Fair), gaps, and 11-target window snapping. | 🟢 Active |
| **`night-rules`** | Library | `night-core`, `serde`, `ron`, `dirs`, `pinnacle-api` | RON-powered declarative window rules (floating, tag placement) and terminal window swallowing engine. | 🟢 Active |
| **`night-binds`** | Library | `night-core`, `night-tags`, `night-layout`, `pinnacle-api` | Centralized keybindings, Snowcap modals (Launcher, Switcher, Grid Exposé, Palette, OSD, Snip, Flyout). | 🟢 Active |
| **`night-pinnacle`**| Binary | All internal crates + `pinnacle-api`, `anyhow` | Main binary orchestrator running `pinnacle_api::main!(config)` with multi-client async IPC server. | 🟢 Active |
| **`night-ctl`** | Binary | `night-core`, `night-layout`, `clap`, `tokio`, `anyhow` | Lightning-fast CLI client for sub-millisecond desktop control, live event streaming, and status reporting. | 🟢 Active |
| **`night-settings`**| Binary | `night-core`, `slint`, `tokio`, `anyhow` | Pure-Rust Bihar Night Control Center GUI with live gaps sliders, borders, and monitor profiles. | 🟢 Active |
| **`night-session`** | Binary | `night-core`, `clap`, `tokio`, `anyhow` | Ergonomic desktop session supervisor with lock, suspend, logout, and systemd integration. | 🟢 Active |

---

## 🌐 External Ecosystem Integration

| Tool | Ecosystem / GitHub | Night Integration Layer |
| :--- | :--- | :--- |
| **Pinnacle** | [pinnacle-comp/pinnacle](https://github.com/pinnacle-comp/pinnacle) | Wayland Compositor running `night-pinnacle` via `pinnacle.toml`. |
| **Ironbar** | [JakeStanger/ironbar](https://github.com/JakeStanger/ironbar) | Status bar started in `night-daemons`; live layout/tag stream via `nightctl stream`. |
| **Snowcap** | [pinnacle-comp/snowcap](https://github.com/pinnacle-comp/snowcap) | Official first-party layer-shell widget system (Launcher, Switcher, Grid, Palette, OSD, Borders). |
| **Vivi** | [Loafer19/vivi](https://github.com/Loafer19/vivi) | 100% Rust-native modern media player (Slint UI + libmpv) mapped to `Super + m`. |
| **rsclip** | [jR4dh3y/rsclip](https://github.com/jR4dh3y/rsclip) | 100% Rust-native Wayland clipboard manager (`rsclipd`) bound to `Super + v`. |
| **Alacritty** | [alacritty/alacritty](https://github.com/alacritty/alacritty) | Primary terminal bound to `Super + Return` and dropdown scratchpad drawer (`Super + ` ` `). |
| **Rio** | [raphamorim/rio](https://github.com/raphamorim/rio) | Alternative terminal bound to `Super + Shift + Return`. |
| **Favnyr** | [GlitchAwakened/Favnyr](https://github.com/GlitchAwakened/Favnyr) | Rust + Slint UI file manager bound to `Super + e`. |
| **Yazi** | [sxyazi/yazi](https://github.com/sxyazi/yazi) | TUI File manager launched in terminal via `Super + y`. |
| **Zen Browser** | [zen-browser.app](https://zen-browser.app/) | Modern focused web browser bound to `Super + b`. |
| **Brave** | [brave.com](https://brave.com/) | Privacy-focused alternative browser bound to `Super + Shift + b`. |
| **Wired** | [Toqozz/wired-notify](https://github.com/Toqozz/wired-notify) | Notification daemon started in `night-daemons`; native D-Bus integration via `notify-rust`. |
| **SwayOSD** | [ErikReider/SwayOSD](https://github.com/ErikReider/SwayOSD) | Server started in `night-daemons`; paired with in-process Snowcap visual HUD. |
| **swww** | [LGFae/swww](https://github.com/LGFae/swww) | Wallpaper daemon with smooth Bihar Night wipe transitions. |
| **Wayshot + Satty**| [waycrate/wayshot](https://github.com/waycrate/wayshot) + [gabm/Satty](https://github.com/gabm/Satty) | Screenshot and annotation pipeline bound to `Print` and `Super + Shift + s`. |
| **wl-screenrec** | [russelltg/wl-screenrec](https://github.com/russelltg/wl-screenrec) | Hardware-accelerated 60fps Wayland recording bound to `Super + Alt + r`. |
| **wayidle** | [hirsch-conner/wayidle](https://github.com/hirsch-conner/wayidle) | Inactivity monitor paired with smart pure-Rust media-aware idle supervisor. |
| **Veila** | [naurissteins/Veila](https://github.com/naurissteins/Veila) | Pure-Rust Wayland screen locker triggered on idle or via `Super + Escape`. |

---

## 🚀 Workspace Tasks (`cargo xtask`)

```bash
cargo xtask check     # Verifies all 8 crates in the workspace
cargo xtask build     # Builds night-pinnacle in release mode
cargo xtask install   # Compiles, installs binary, links pinnacle.toml, deploys session & theme
cargo xtask doctor    # Runs comprehensive diagnostics on all 18 tools and environment
cargo xtask session   # Installs night.desktop for login managers (GDM, SDDM, Ly, greetd)
cargo xtask theme     # Generates Bihar Night styling for Alacritty, Ironbar, Anyrun, Veila
cargo xtask run       # Builds and runs Pinnacle compositor
cargo xtask msg <CMD> # IPC control (status, reload, switch-tag, layout, gaps, spawn, close)
cargo xtask uninstall # Removes binary and symlink
```

---

## 📝 Implementation Progress Log

| Date | Subsystem | Action | Details |
| :--- | :--- | :--- | :--- |
| **2026-11-03** | Format & Tools | xtask, RON, TOML, postcard | Implemented `cargo xtask` runner, TOML user config, RON window rules, and postcard binary session caching. |
| **2026-11-03** | Installation | Removed Stow & Shell Scripts | Replaced legacy Stow/Makefiles with `cargo xtask install`. |
| **2026-11-03** | Architecture | Workspace Reorganization | Refactored into a 7-crate root workspace. |
| **2026-11-08** | Visual Identity | Phase 26: Bihar Night Theme Engine | Rebranded Tokyo Night to Bihar Night (Ganga Midnight, Rajgir Mist, Bodh Gaya Serenity, Nalanda Dawn) with 5K UHD wallpapers (`bihar-night.jpg`) and `BIHAR_NIGHT.md`. |
| **2026-11-08** | Diurnal, Power, Layout | Phase 27: Scheduler, Power Saver, Monocle | Added Diurnal Solar Theme Scheduler (`diurnal_watcher.rs`), Battery Conservation Mode (`power_watcher.rs`), Monocle Layout Engine (`LayoutKind::Monocle`, `LayoutProfile::Focus`), and full CLI/Slint GUI integration. |
| **2026-11-25** | Launcher, Daemons, CLI | Phase 28: Walker + Elephant, Daemon Supervisor, `nightctl doctor` | Integrated Walker + Elephant as primary launcher with live Bihar Night CSS sync, added `supervise_core_daemons()` + `GetDaemons` IPC, `nightctl daemons`, `nightctl doctor`, and atomic state/config persistence. |
| **2026-11-26** | IPC & Session Restore | Phase 29: `interprocess` IPC & Persistent Session Recovery | Migrated IPC to `interprocess::local_socket::tokio` + `LengthDelimitedCodec`, moved `SavedSession` to `$XDG_DATA_HOME/night/saved-session.bin`, added `resolve_launch_command()`, per-window tag/layout restore, and `nightctl session status`. |
| **2026-11-26** | Abstractions & Tests | Phase 30: Modal/Wallpaper Traits & Pure System Parsers | Added `night-binds::modal` shared scaffolding across 8 modals, `WallpaperBackend` trait (`SwwwBackend`/`SwaybgBackend`), and pure parsers with unit tests in `audio.rs`, `power.rs`, and `night-rules`. |

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
| **`xtask`** | Binary | `night-core`, `tokio`, `dirs` | Native CLI runner for build, install, check, and IPC remote control (`cargo xtask msg`). | 🟢 Active |
| **`night-core`** | Library | `dirs`, `interprocess`, `which`, `shell-words`, `postcard`, `toml`, `serde` | Theme constants, default apps, TOML config, binary session caching, and Unix socket IPC client/server. | 🟢 Active |
| **`night-daemons`** | Library | `night-core`, `which`, `pinnacle-api` | Supervises background services (`swww`, `ironbar`, `wired`, `swayosd`, `wayidle`) with `$PATH` discovery. | 🟢 Active |
| **`night-tags`** | Library | `night-core`, `pinnacle-api` | Multi-monitor dynamic tag management ([1..9]) with runtime display hotplug detection. | 🟢 Active |
| **`night-rules`** | Library | `night-core`, `serde`, `ron`, `dirs`, `pinnacle-api` | RON-powered declarative window matching rules (auto-floating dialogs, tag assignments). | 🟢 Active |
| **`night-binds`** | Library | `night-core`, `night-tags`, `pinnacle-api` | Centralized keybindings for apps, directional focus, swap, fullscreen, OSD, screenshots, and mouse drag/resize. | 🟢 Active |
| **`night-pinnacle`**| Binary | All internal crates + `pinnacle-api` | Main binary orchestrator running `pinnacle_api::main!(config)` with async IPC server. | 🟢 Active |

---

## 🌐 External Ecosystem Integration

| Tool | Ecosystem / GitHub | Night Integration Layer |
| :--- | :--- | :--- |
| **Pinnacle** | [pinnacle-comp/pinnacle](https://github.com/pinnacle-comp/pinnacle) | Wayland Compositor running `night-pinnacle` via `pinnacle.toml`. |
| **Ironbar** | [JakeStanger/ironbar](https://github.com/JakeStanger/ironbar) | Status bar started in `night-daemons`. |
| **Snowcap** | [pinnacle-comp/snowcap](https://github.com/pinnacle-comp/snowcap) | Alternative widget/bar system by Pinnacle team. |
| **Anyrun** | [anyrun-org/anyrun](https://github.com/anyrun-org/anyrun) | Modular launcher bound to `Super + Space` in `night-binds`. |
| **Yofi** | [loki-47-500/yofi](https://github.com/loki-47-500/yofi) | Minimalist pure-Rust launcher bound to `Super + d` in `night-binds`. |
| **Alacritty** | [alacritty/alacritty](https://github.com/alacritty/alacritty) | Primary terminal bound to `Super + Return` in `night-binds`. |
| **Rio** | [raphamorim/rio](https://github.com/raphamorim/rio) | Alternative terminal bound to `Super + Shift + Return`. |
| **Yazi** | [sxyazi/yazi](https://github.com/sxyazi/yazi) | TUI File manager launched in terminal via `Super + e`. |
| **Wired** | [Toqozz/wired-notify](https://github.com/Toqozz/wired-notify) | Notification daemon started in `night-daemons`. |
| **SwayOSD** | [ErikReider/SwayOSD](https://github.com/ErikReider/SwayOSD) | Server started in `night-daemons`; volume/brightness keys mapped in `night-binds`. |
| **swww** | [LGFae/swww](https://github.com/LGFae/swww) | Wallpaper daemon started in `night-daemons`. |
| **Wayshot + Satty**| [waycrate/wayshot](https://github.com/waycrate/wayshot) + [gabm/Satty](https://github.com/gabm/Satty) | Screenshot and annotation pipeline bound to `Print` and `Super + Print`. |
| **wl-screenrec** | [russelltg/wl-screenrec](https://github.com/russelltg/wl-screenrec) | Hardware-accelerated recording bound to `Super + Alt + r`. |
| **wayidle** | [hirsch-conner/wayidle](https://github.com/hirsch-conner/wayidle) | Idle monitor started in `night-daemons`. |
| **Veila** | [naurissteins/Veila](https://github.com/naurissteins/Veila) | Pure-Rust Wayland screen locker triggered on idle or via `Super + Escape`. |

---

## 🚀 Workspace Tasks (`cargo xtask`)

```bash
cargo xtask check     # Verifies all 7 crates in the workspace
cargo xtask build     # Builds night-pinnacle in release mode
cargo xtask install   # Compiles, installs binary to ~/.cargo/bin, links pinnacle.toml & sets up ~/.config/night/
cargo xtask run       # Builds and runs Pinnacle compositor
cargo xtask uninstall # Removes binary and symlink
```

---

## 📝 Implementation Progress Log

| Date | Subsystem | Action | Details |
| :--- | :--- | :--- | :--- |
| **2026-11-03** | Format & Tools | xtask, RON, TOML, postcard | Implemented `cargo xtask` runner, TOML user config, RON window rules, and postcard binary session caching. |
| **2026-11-03** | Installation | Removed Stow & Shell Scripts | Replaced legacy Stow/Makefiles with `cargo xtask install`. |
| **2026-11-03** | Architecture | Workspace Reorganization | Refactored into a 7-crate root workspace. |

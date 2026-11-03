# 🌙 Night
> A modular, high-performance Rust configuration and desktop suite for the [Pinnacle](https://github.com/pinnacle-comp/pinnacle) Wayland compositor.

**Night** is a minimalist, keyboard-driven Wayland environment designed for high-focus development during late-night hours. It provides a modular, async Rust architecture for configuring Pinnacle while orchestrating a suite of external Rust-native desktop tools.

---

## 💎 The Four Pillars: Purpose-Driven Architecture

| Tool / Format | Best Use Case in Night | Location / Artifact |
| :--- | :--- | :--- |
| 🦀 **`cargo xtask`** | **Pure-Rust Automation:** Replaces Makefiles and shell scripts with a type-safe CLI task runner. | [`xtask/`](./xtask) (`cargo xtask install`) |
| 📄 **TOML** | **Human-Editable Preferences:** Clean key-value format for desktop settings and app defaults. | [`config/night.toml`](./config/night.toml) $\rightarrow$ `~/.config/night/night.toml` |
| 🦀 **RON** | **Rich Rust Types & Enums:** Strongly typed representation for complex window matching rules and actions. | [`config/rules.ron`](./config/rules.ron) $\rightarrow$ `~/.config/night/rules.ron` |
| ⚡ **`postcard`** | **Ultra-Fast Binary Session Caching:** Zero-allocation binary serde format for instant state restoration across reloads. | `$XDG_RUNTIME_DIR/night-session.bin` |

---

## 🏗️ Architecture: The `night` Workspace

All source code lives in a clean, root-level **Cargo Workspace** (7 crates):

```text
night/
├── Cargo.toml                    # Root Cargo Workspace manifest
├── Cargo.lock
├── pinnacle.toml                 # Runner manifest (tells Pinnacle how to start Night)
│
├── config/                       # User-facing default configurations
│   ├── night.toml                # TOML: Human-editable settings & default apps
│   └── rules.ron                 # RON: Strongly-typed window rules & enum actions
│
├── xtask/                        # Pure-Rust workspace task runner (`cargo xtask`)
│   ├── Cargo.toml
│   └── src/main.rs
│
└── crates/                       # Production desktop crates
    ├── night-core/               # TOML config parsing, postcard state caching, theme constants
    ├── night-daemons/            # Autostart supervisor for background services
    ├── night-tags/               # AwesomeWM-style dynamic tag management ([1..9])
    ├── night-rules/              # RON-powered declarative window placement & auto-floating
    ├── night-binds/              # Keyboard shortcuts, media keys, and screenshot binds
    └── night-pinnacle/           # Main binary entrypoint orchestrating all crates
```

---

## 🛠️ Recommended External Rust Ecosystem

Night acts as the brain of your desktop, coordinating with standalone, high-performance Rust tools that you install from GitHub or package managers:

| Role | Tool | Tech Stack | How Night Integrates |
| :--- | :--- | :--- | :--- |
| **Compositor Engine** | [Pinnacle](https://github.com/pinnacle-comp/pinnacle) | Rust (`smithay`) | The Wayland server running `night-pinnacle`. |
| **Status Bar** | [Ironbar](https://github.com/JakeStanger/ironbar) | GTK4-rs + Layer-Shell | Launched by `night-daemons` on startup. |
| **Native Widgets** | [Snowcap](https://github.com/pinnacle-comp/snowcap) | Rust + Wayland | Companion widget system by the Pinnacle team. |
| **Alternative Widgets** | [Eww](https://github.com/elkowar/eww) | GTK-rs + Cairo | Can be launched by `night-daemons`. |
| **App Launcher** | [Anyrun](https://github.com/anyrun-org/anyrun) | GTK4-rs + Layer-Shell | Bound to `Super + Space` in `night-binds`. |
| **Minimalist Launcher** | [Yofi](https://github.com/loki-47-500/yofi) | Pure Rust (`smithay-client-toolkit`) | Bound to `Super + d` in `night-binds`. |
| **Terminal Emulator** | [Alacritty](https://github.com/alacritty/alacritty) | Rust + OpenGL | Primary terminal bound to `Super + Return`. |
| **Alternative Terminal**| [Rio](https://github.com/raphamorim/rio) | Rust + WebGPU | Bound to `Super + Shift + Return`. |
| **File Manager** | [Yazi](https://github.com/sxyazi/yazi) | Rust + Tokio | Bound to `Super + e` (in Alacritty). |
| **Notifications** | [Wired](https://github.com/Toqozz/wired-notify) | Pure Rust | Launched by `night-daemons` on startup. |
| **OSD Overlay** | [SwayOSD](https://github.com/ErikReider/SwayOSD) | GTK4-rs + Layer-Shell | Server started by `night-daemons`; volume/brightness keys mapped in `night-binds`. |
| **Wallpaper Daemon** | [swww](https://github.com/LGFae/swww) | Pure Rust | Started by `night-daemons` for smooth wallpaper transitions. |
| **Screenshots** | [Wayshot](https://github.com/waycrate/wayshot) + [Satty](https://github.com/gabm/Satty) | Rust (`wayland-rs` / GTK4) | Bound to `Print` and `Super + Print` in `night-binds`. |
| **Screen Recorder** | [wl-screenrec](https://github.com/russelltg/wl-screenrec) | Rust + FFmpeg | Bound to `Super + Alt + r` in `night-binds`. |
| **Idle Management** | [wayidle](https://github.com/hirsch-conner/wayidle) | Pure Rust | Screen lock & DPMS monitor started by `night-daemons`. |

---

## ⌨️ Default Keybindings (in `night-binds`)

* `Super + Return` $\rightarrow$ Launch **Alacritty**
* `Super + Shift + Return` $\rightarrow$ Launch **Rio**
* `Super + Space` $\rightarrow$ Launch **Anyrun**
* `Super + d` $\rightarrow$ Launch **Yofi**
* `Super + e` $\rightarrow$ Launch **Yazi** (Terminal File Manager)
* `Super + Shift + q` $\rightarrow$ Close Focused Window
* `Super + Shift + r` $\rightarrow$ Reload Pinnacle Configuration
* `Super + Shift + e` $\rightarrow$ Exit Pinnacle Session
* `Print` $\rightarrow$ Interactive Region Screenshot & Annotate (**Wayshot** + **Satty**)
* `Super + Print` $\rightarrow$ Fullscreen Screenshot to `~/Pictures`
* `Super + Alt + r` $\rightarrow$ Toggle Screen Recording (**wl-screenrec**)
* `Super + [1..9]` $\rightarrow$ View Tag 1–9
* `Super + Shift + [1..9]` $\rightarrow$ Move Window to Tag 1–9
* `XF86Audio*` / `XF86MonBrightness*` $\rightarrow$ SwayOSD Volume & Brightness Controls

---

## 🚀 Installation via `cargo xtask`

No need for GNU Stow, Makefiles, or shell scripts. Everything is handled by the pure-Rust task runner:

### 1. Install System Dependencies (Arch Linux)
```bash
sudo pacman -S --needed rustup base-devel git slurp
yay -S pinnacle-comp-git alacritty ironbar-bin anyrun-git wired-notify-bin swayosd-git swww wayshot satty-bin yazi
```

### 2. Install Night
```bash
git clone https://github.com/your-username/night.git
cd night

# Run the pure-Rust xtask installer
cargo xtask install
```

`cargo xtask install` automatically:
1. Compiles `night-pinnacle` in release mode.
2. Installs the binary to `~/.cargo/bin/night-pinnacle`.
3. Links `pinnacle.toml` to `~/.config/pinnacle/pinnacle.toml`.
4. Initializes default user configs (`~/.config/night/night.toml` & `~/.config/night/rules.ron`).

Start your session with:
```bash
pinnacle
```

---

## 🔧 xtask Commands

```bash
cargo xtask check     # Run cargo check across all workspace crates
cargo xtask build     # Build release binary
cargo xtask install   # Complete setup into ~/.cargo/bin and ~/.config/
cargo xtask run       # Build and start Pinnacle directly
cargo xtask uninstall # Cleanly remove installed binary and symlink
```

---

## 📂 Project Tracking Docs
* 🗺️ **[ROADMAP.md](./ROADMAP.md)**: Implementation phases, milestones, and deliverables.
* 🛠️ **[IMPLEMENTATION.md](./IMPLEMENTATION.md)**: Technical specifications, workspace crate details, and status tracker.

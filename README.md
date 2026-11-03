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
│   ├── rules.ron                 # RON: Strongly-typed window rules & enum actions
│   └── monitors.ron              # RON: Display connectors, HiDPI scaling, modes & VRR
│
├── xtask/                        # Pure-Rust workspace task runner (`cargo xtask`)
│   ├── Cargo.toml
│   └── src/main.rs
│
├── packaging/                    # Arch Linux & AUR packaging specifications
│   └── PKGBUILD
│
└── crates/                       # Production desktop crates
    ├── night-core/               # TOML config parsing, postcard state caching, theme constants
    ├── night-daemons/            # Autostart supervisor for background services
    ├── night-tags/               # AwesomeWM-style dynamic tag management ([1..9])
    ├── night-layout/             # Dynamic layout engine (MasterStack, Dwindle, Spiral, Fair)
    ├── night-rules/              # RON-powered declarative window placement & auto-floating
    ├── night-binds/              # Keyboard shortcuts, media keys, and screenshot binds
    ├── night-ctl/                # High-performance CLI client & real-time status streaming tool
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
| **GUI File Manager** | [Favnyr](https://github.com/GlitchAwakened/Favnyr) | Rust + Slint UI | Modern, lightweight native file manager bound to `Super + e`. |
| **Terminal File Manager**| [Yazi](https://github.com/sxyazi/yazi) | Rust + Tokio | Blazingly fast async TUI file manager bound to `Super + y`. |
| **Primary Web Browser** | [Zen Browser](https://zen-browser.app/) | Gecko / Fast & Modern | Focused, privacy-conscious browser bound to `Super + b`. |
| **Alternative Browser** | [Brave](https://brave.com/) | Chromium | Privacy browser bound to `Super + Shift + b`. |
| **Notifications** | [Wired](https://github.com/Toqozz/wired-notify) | Pure Rust | Launched by `night-daemons` on startup. |
| **OSD Overlay** | [SwayOSD](https://github.com/ErikReider/SwayOSD) | GTK4-rs + Layer-Shell | Server started by `night-daemons`; volume/brightness keys mapped in `night-binds`. |
| **Wallpaper Daemon** | [swww](https://github.com/LGFae/swww) | Pure Rust | Started by `night-daemons` for smooth wallpaper transitions. |
| **Screenshots** | [Wayshot](https://github.com/waycrate/wayshot) + [Satty](https://github.com/gabm/Satty) | Rust (`wayland-rs` / GTK4) | Bound to `Print` and `Super + Print` in `night-binds`. |
| **Screen Recorder** | [wl-screenrec](https://github.com/russelltg/wl-screenrec) | Rust + FFmpeg | Bound to `Super + Alt + r` in `night-binds`. |
| **Idle Management** | [wayidle](https://github.com/hirsch-conner/wayidle) | Pure Rust | Screen lock & DPMS monitor started by `night-daemons`. |
| **Screen Locker** | [Veila](https://github.com/naurissteins/Veila) | Pure Rust (`ext-session-lock-v1`) | Fast, modern PAM screen locker triggered on idle or via `Super + Escape`. |

---

## ⌨️ Default Keybindings & Mouse Controls (in `night-binds`)

### Application Launchers
* `Super + Return` $\rightarrow$ Launch **Alacritty** (Terminal)
* `Super + Shift + Return` $\rightarrow$ Launch **Rio** (GPU Terminal)
* `Super + Space` $\rightarrow$ Launch **Anyrun** (Runner)
* `Super + d` $\rightarrow$ Launch **Yofi** (Minimal Pure-Rust Launcher)
* `Super + e` $\rightarrow$ Launch **Favnyr** (Rust+Slint GUI File Manager)
* `Super + y` $\rightarrow$ Launch **Yazi** (Async Terminal File Manager)
* `Super + b` $\rightarrow$ Launch **Zen Browser** (Primary Web Browser)
* `Super + Shift + b` $\rightarrow$ Launch **Brave Browser** (Alternative Browser)
* `Super + ` ` ` (tilde) $\rightarrow$ Toggle **Scratchpad** Dropdown Terminal
* `Super + Escape` $\rightarrow$ Lock Session (**Veila** - Pure Rust Locker)

### Window Navigation & Layout
* `Super + Tab` $\rightarrow$ Cycle Tiling Layout (**MasterStack** $\rightarrow$ **Dwindle** $\rightarrow$ **Spiral** $\rightarrow$ **Fair**)
* `Super + =` / `Super + -` $\rightarrow$ Increase / Decrease Layout Gaps in Real-Time
* `Super + [h/j/k/l]` or `Super + Arrows` $\rightarrow$ Directional Focus (West / South / North / East)
* `Super + Shift + [h/j/k/l]` $\rightarrow$ Swap Window with Neighbor in Direction
* `Super + f` $\rightarrow$ Toggle Fullscreen Mode
* `Super + Shift + Space` $\rightarrow$ Toggle Floating Mode
* `Super + s` $\rightarrow$ Toggle Sticky Window (pin across all workspace tags)
* `Super + p` $\rightarrow$ Toggle Picture-in-Picture (PiP) Floating Overlay Mode
* `Super + c` $\rightarrow$ Center Focused Floating Window on Active Display
* `Super + Shift + q` $\rightarrow$ Close Focused Window
* `Super + Left-Click + Drag` $\rightarrow$ Interactive Floating Window Move
* `Super + Right-Click + Drag` $\rightarrow$ Interactive Floating Window Resize

### Dynamic Workspaces (AwesomeWM Style with Per-Output Isolation)
* `Super + [1..9]` $\rightarrow$ View Tag 1–9 on Focused Monitor
* `Super + Shift + [1..9]` $\rightarrow$ Move Window to Tag 1–9 on Focused Monitor

### Multi-Monitor Navigation & Screen Power
* `Super + o` or `Super + .` (period) $\rightarrow$ Focus Next Connected Monitor
* `Super + ,` (comma) $\rightarrow$ Focus Previous Connected Monitor
* `Super + Shift + o` or `Super + Shift + .` $\rightarrow$ Move Focused Window to Next Monitor
* `Super + Shift + ,` $\rightarrow$ Move Focused Window to Previous Monitor
* `Super + Shift + Escape` $\rightarrow$ Toggle DPMS Screen Power (Screen Sleep / Wake)

### Multimedia, Screenshots & Session
* `Super + /` $\rightarrow$ Open **Snowcap** Keybinds Cheatsheet Overlay (Tokyo Night themed)
* `Super + BackSpace` $\rightarrow$ Open **Snowcap** Tokyo Night Session Flyout Menu (Lock, Sleep, Reboot, Power Off)
* `Super + Shift + e` $\rightarrow$ Open **Snowcap** Quit Confirmation Prompt
* `Super + Shift + r` $\rightarrow$ Reload Pinnacle Configuration
* `Super + Shift + x` $\rightarrow$ Instant Exit Pinnacle Session
* `Print` $\rightarrow$ Interactive Region Screenshot & Annotate (**Wayshot** + **Satty**)
* `Super + Print` $\rightarrow$ Fullscreen Screenshot to `~/Pictures`
* `Super + Alt + r` $\rightarrow$ Toggle Screen Recording (**wl-screenrec**)
* `XF86Audio*` / `XF86MonBrightness*` $\rightarrow$ SwayOSD Volume & Brightness Indicators

---

## ⚡ Instant Remote Control & Live Status Streaming (`nightctl`)

Night provides a dedicated, blazing-fast pure-Rust CLI tool `nightctl` installed to `~/.cargo/bin/nightctl` for sub-millisecond IPC communication, window management, and real-time event streaming:

```bash
# Query comprehensive desktop status (active tag, layout, gaps, monitors, focused window)
nightctl status
nightctl status --json

# Stream real-time status updates without polling lag (ideal for Ironbar, Waybar, or scripts)
nightctl stream
nightctl stream --format "󰕰 {layout} | 󱂬 {active_tag} | {active_window}"

# Control workspace tags and window tiling
nightctl tag 2
nightctl layout next
nightctl gaps +2
nightctl gaps -2

# Multi-monitor display management & window moving
nightctl output next
nightctl output prev
nightctl output focus DP-1
nightctl output move-window next

# DPMS display power management
nightctl dpms toggle
nightctl dpms off
nightctl dpms on

# Advanced window operations (sticky across tags, picture-in-picture, center)
nightctl window sticky
nightctl window pip
nightctl window center
nightctl window float

# Toggle dropdown scratchpad terminal and Snowcap modals
nightctl scratchpad
nightctl overlay
nightctl quit-prompt
nightctl session-menu
```

Alternatively, remote commands can be delivered via `cargo xtask msg <CMD>`:
* `cargo xtask msg status` $\rightarrow$ Prints detailed desktop status.
* `cargo xtask msg stream` $\rightarrow$ Streams live compositor events.
* `cargo xtask msg switch-tag <TAG>` $\rightarrow$ Switch to tag `[1..9]`.
* `cargo xtask msg layout [next]` $\rightarrow$ Query or cycle tiling layouts.
* `cargo xtask msg gaps <delta>` $\rightarrow$ Adjust gaps by numeric delta (e.g. `2.0` or `-2.0`).
* `cargo xtask msg scratchpad` $\rightarrow$ Toggles Quake dropdown terminal.
* `cargo xtask msg overlay` $\rightarrow$ Triggers Snowcap keybind overlay.
* `cargo xtask msg quit-prompt` $\rightarrow$ Triggers Snowcap quit prompt.
* `cargo xtask msg reload` $\rightarrow$ Reloads compositor configuration.
* `cargo xtask msg close` $\rightarrow$ Closes currently focused window.
* `cargo xtask msg spawn <CMD>` $\rightarrow$ Spawns a command string through Night.

---

## 🩺 System Diagnostics (`cargo xtask doctor`)

Run the pure-Rust environment doctor at any time to verify system health, config files, and Wayland session environment:

```bash
cargo xtask doctor
```
Outputs a live checklist of all 18 tools in the 100% Rust suite, highlighting any missing packages and printing a single `yay -S --needed ...` command to install them.

---

## 🚀 Installation via `cargo xtask`

No need for GNU Stow, Makefiles, or shell scripts. Everything is handled by the pure-Rust task runner:

### 1. Install System Dependencies (Arch Linux)
```bash
sudo pacman -S --needed rustup base-devel git slurp
paru -S pinnacle-comp-git alacritty ironbar-bin anyrun-git wired-notify-bin swayosd-git swww wayshot satty-bin yazi veila-bin favnyr-git zen-browser-bin brave-bin
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

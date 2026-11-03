//! System default programs and binaries for the Night Desktop Environment

pub struct Defaults;

impl Defaults {
    /// Primary GPU-accelerated terminal emulator
    pub const TERMINAL: &'static str = "alacritty";

    /// Alternative WebGPU terminal emulator
    pub const ALT_TERMINAL: &'static str = "rio";

    /// Primary modular application runner
    pub const LAUNCHER: &'static str = "anyrun";

    /// Minimalist pure-Rust application launcher
    pub const MINIMAL_LAUNCHER: &'static str = "yofi";

    /// Modern lightweight Rust+Slint GUI file manager
    pub const FILE_MANAGER: &'static str = "favnyr";

    /// Blazingly fast asynchronous terminal file manager
    pub const TERM_FILE_MANAGER: &'static str = "yazi";

    /// Modern focused web browser
    pub const BROWSER: &'static str = "zen-browser";

    /// Privacy-focused alternative browser
    pub const ALT_BROWSER: &'static str = "brave";

    /// Modern lightweight Rust+Slint media player
    pub const MEDIA_PLAYER: &'static str = "vivi";

    /// Pure-Rust Wayland clipboard manager and daemon
    pub const CLIPBOARD_DAEMON: &'static str = "rsclipd";
    pub const CLIPBOARD_MANAGER: &'static str = "rsclip";

    /// Pure-Rust Wayland screen locker
    pub const LOCKER: &'static str = "veila";

    /// Screenshot tool
    pub const SCREENSHOT_CLI: &'static str = "wayshot";

    /// Screenshot annotation editor
    pub const ANNOTATOR: &'static str = "satty";

    /// Hardware-accelerated screen recorder
    pub const SCREEN_RECORDER: &'static str = "wl-screenrec";

    /// Notification daemon
    pub const NOTIFICATION_DAEMON: &'static str = "wired";

    /// Status bar
    pub const STATUS_BAR: &'static str = "ironbar";

    /// OSD service
    pub const OSD_SERVER: &'static str = "swayosd-server";
    pub const OSD_CLIENT: &'static str = "swayosd-client";

    /// Wallpaper daemon
    pub const WALLPAPER_DAEMON: &'static str = "swww-daemon";
}


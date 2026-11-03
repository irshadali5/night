#!/usr/bin/env bash
# 🌙 Project Night: Hardware-Accelerated Screen Recorder
# 100% Rust-native Wayland recording toggle using wl-screenrec.

set -euo pipefail

VIDEOS_DIR="${XDG_VIDEOS_DIR:-$HOME/Videos}/Recordings"
mkdir -p "$VIDEOS_DIR"

# Check if wl-screenrec is currently running; if so, stop it
if pgrep -x "wl-screenrec" > /dev/null 2>&1; then
    pkill -SIGINT -x "wl-screenrec"
    if command -v notify-send > /dev/null 2>&1; then
        notify-send -a "Night Desktop" "Screen Recording Stopped" "Video saved to $VIDEOS_DIR"
    fi
    exit 0
fi

TIMESTAMP=$(date +"%Y-%m-%d_%H-%M-%S")
TARGET_FILE="$VIDEOS_DIR/Recording_${TIMESTAMP}.mp4"

if command -v notify-send > /dev/null 2>&1; then
    notify-send -a "Night Desktop" "Screen Recording Started" "Capturing to $TARGET_FILE (Press Super+Alt+R to stop)"
fi

# Launch hardware-accelerated recording with audio capture support
wl-screenrec --audio --encode-resolution 1920x1080 -f "$TARGET_FILE"


#!/usr/bin/env bash
# 🌙 Project Night: Screenshot Tool
# High-performance 100% Rust-native Wayland screen capture using wayshot, slurp, and satty.

set -euo pipefail

PICTURES_DIR="${XDG_PICTURES_DIR:-$HOME/Pictures}/Screenshots"
mkdir -p "$PICTURES_DIR"

TIMESTAMP=$(date +"%Y-%m-%d_%H-%M-%S")
TARGET_FILE="$PICTURES_DIR/Screenshot_${TIMESTAMP}.png"
MODE="${1:-area}"

case "$MODE" in
    area)
        # Interactive selection with slurp, streamed into satty annotation editor
        GEOM=$(slurp -b "#1a1b26cc" -c "#7aa2f7" -w 2 -s "#7aa2f733")
        if [[ -n "$GEOM" ]]; then
            wayshot -s "$GEOM" --stdout | satty --filename - --output-filename "$TARGET_FILE"
        fi
        ;;
    full)
        # Fullscreen immediate capture to file and clipboard
        wayshot -f "$TARGET_FILE"
        if command -v wl-copy > /dev/null 2>&1; then
            wl-copy < "$TARGET_FILE"
        fi
        if command -v wired > /dev/null 2>&1 || command -v notify-send > /dev/null 2>&1; then
            notify-send -a "Night Desktop" "Screenshot Saved" "Captured to $TARGET_FILE and copied to clipboard."
        fi
        ;;
    window)
        # Active window capture
        wayshot -s "$(slurp)" --stdout | satty --filename - --output-filename "$TARGET_FILE"
        ;;
    *)
        echo "Usage: $0 [area|full|window]"
        exit 1
        ;;
esac


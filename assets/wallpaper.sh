#!/usr/bin/env bash
# 🌙 Project Night: Dynamic Wallpaper Controller
# Pure-Rust Wayland wallpaper management via swww with Bihar Night transition shaders.

set -euo pipefail

# Ensure swww-daemon is running
if ! pgrep -x "swww-daemon" > /dev/null 2>&1; then
    swww-daemon &
    sleep 0.5
fi

# Bihar Night Ganga Midnight solid color fallback (#1a1b26)
FALLBACK_COLOR="1a1b26"

WALLPAPER_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/night/wallpapers"
ALT_DIR="$HOME/Pictures/Wallpapers"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_WALLPAPERS="$SCRIPT_DIR/wallpapers"

# 1. Specific image passed as argument
if [[ $# -ge 1 && -f "$1" ]]; then
    TARGET_IMG="$1"
else
    # 2. Check for signature Bihar Night wallpapers first
    TARGET_IMG=""
    for candidate in \
        "$WALLPAPER_DIR/bihar-night.jpg" \
        "$WALLPAPER_DIR/bihar-night-ganga.jpg" \
        "$WALLPAPER_DIR/bihar-night-bodhgaya.jpg" \
        "$REPO_WALLPAPERS/bihar-night.jpg" \
        "$REPO_WALLPAPERS/bihar-night-ganga.jpg"; do
        if [[ -f "$candidate" ]]; then
            TARGET_IMG="$candidate"
            break
        fi
    done

    # 3. Otherwise pick from wallpaper directories
    if [[ -z "$TARGET_IMG" ]]; then
        for dir in "$WALLPAPER_DIR" "$REPO_WALLPAPERS" "$ALT_DIR"; do
            if [[ -d "$dir" ]]; then
                FOUND=$(find "$dir" -maxdepth 1 -type f \( -name "*.png" -o -name "*.jpg" -o -name "*.jpeg" -o -name "*.webp" \) | shuf -n 1)
                if [[ -n "$FOUND" ]]; then
                    TARGET_IMG="$FOUND"
                    break
                fi
            fi
        done
    fi
fi

if [[ -n "$TARGET_IMG" && -f "$TARGET_IMG" ]]; then
    echo "🌙 Setting Bihar Night wallpaper: $TARGET_IMG"
    swww img "$TARGET_IMG" \
        --transition-type grow \
        --transition-pos "0.5,0.5" \
        --transition-step 90 \
        --transition-fps 60 \
        --transition-duration 1.5
else
    echo "🌙 No wallpaper image found; clearing to solid Bihar Night (#${FALLBACK_COLOR})"
    swww clear "$FALLBACK_COLOR"
fi


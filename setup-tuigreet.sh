#!/bin/bash
# 🌙 Rust-Native TUI Display Manager Setup Script
# Replaces cosmic-greeter with greetd-tuigreet (Ratatui / Rust) styled with Tokyo Night.

set -e

if [ "$EUID" -ne 0 ]; then
    echo "❌ Error: Please execute this setup script with sudo:"
    echo "   sudo ./setup-tuigreet.sh"
    exit 1
fi

echo "🌙 Setting up 100% Rust-Native TUI Display Manager (greetd + tuigreet)..."

# 1. Stop and disable cosmic-greeter
echo "==> Disabling cosmic-greeter.service..."
systemctl stop cosmic-greeter.service 2>/dev/null || true
systemctl disable cosmic-greeter.service 2>/dev/null || true
systemctl disable display-manager.service 2>/dev/null || true

# 2. Install greetd & greetd-tuigreet
echo "==> Installing greetd-tuigreet via pacman..."
pacman -S --needed --noconfirm greetd greetd-tuigreet

# 3. Create cache directory for tuigreet remembering last user and session
echo "==> Setting up /var/cache/tuigreet..."
mkdir -p /var/cache/tuigreet
chown -R greeter:greeter /var/cache/tuigreet
chmod 755 /var/cache/tuigreet

# 4. Deploy Tokyo Night tuigreet configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
echo "==> Deploying Tokyo Night /etc/greetd/config.toml..."
mkdir -p /etc/greetd
cp "${SCRIPT_DIR}/assets/greetd-tuigreet.toml" /etc/greetd/config.toml

# 5. Enable greetd.service as primary display manager
echo "==> Enabling greetd.service..."
systemctl enable greetd.service

# 6. Verify and update wayland sessions
echo "==> Updating Wayland desktop entry in /usr/share/wayland-sessions/..."
cp "${SCRIPT_DIR}/assets/night.desktop" /usr/share/wayland-sessions/night.desktop
cp "${SCRIPT_DIR}/assets/night.service" /usr/lib/systemd/user/night.service
cp "${SCRIPT_DIR}/assets/night-shutdown.target" /usr/lib/systemd/user/night-shutdown.target

echo "==> Installed Wayland Sessions:"
ls -la /usr/share/wayland-sessions/

# 7. Remove cosmic-greeter
if pacman -Qq cosmic-greeter >/dev/null 2>&1; then
    echo "==> Removing cosmic-greeter..."
    pacman -Rdd --noconfirm cosmic-greeter 2>/dev/null || true
fi

echo "================================================================================"
echo "✨ Rust-native TUI display manager (tuigreet) successfully installed & configured!"
echo "   Tokyo Night TUI login is active on VT 1."
echo "   Available sessions: Night (Pinnacle) and Niri."
echo "================================================================================"

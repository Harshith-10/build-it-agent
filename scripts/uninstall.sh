#!/usr/bin/env bash
set -euo pipefail

# Convenience uninstaller for macOS (per-user LaunchAgent)
# - Stops and disables the LaunchAgent
# - Removes user LaunchAgent plist
# - Removes binary and symlink
# - Removes shared support files
# - Removes package receipt
#

if [[ "$(uname -s || echo _)" != "Darwin" ]]; then
  echo "This uninstaller currently supports macOS only." >&2
  exit 2
fi

APP_NAME="build-it-agent"
IDENTIFIER="com.build-it.agent"
BIN_SYMLINK="/usr/local/bin/${APP_NAME}"
APP_DIR="/usr/local/libexec/${APP_NAME}"
SUPPORT_DIR="/Library/Application Support/${APP_NAME}"
PLIST_PATH="$HOME/Library/LaunchAgents/${IDENTIFIER}.plist"
USER_UID=$(id -u)

# Stop and disable LaunchAgent
set +e
launchctl bootout gui/$USER_UID/${IDENTIFIER} 2>/dev/null
launchctl disable gui/$USER_UID/${IDENTIFIER} 2>/dev/null
set -e

# Remove LaunchAgent plist
if [[ -f "$PLIST_PATH" ]]; then
  rm -f "$PLIST_PATH"
fi

# Remove files
sudo rm -f "$BIN_SYMLINK" 2>/dev/null || true
if [[ -d "$APP_DIR" ]]; then
  sudo rm -rf "$APP_DIR"
fi
if [[ -d "$SUPPORT_DIR" ]]; then
  sudo rm -rf "$SUPPORT_DIR"
fi

# Remove receipt (optional)
if pkgutil --pkg-info "$IDENTIFIER" >/dev/null 2>&1; then
  sudo pkgutil --forget "$IDENTIFIER" || true
fi

echo "Uninstall complete."

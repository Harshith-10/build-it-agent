#!/usr/bin/env bash
set -euo pipefail

# Convenience installer for macOS
# - Builds the release binary
# - Creates the .pkg
# - Installs it via the macOS Installer
#
# Options:
#   --skip-build     Skip building the Rust binary (use existing target/release)
#   --universal      Build a universal binary (x86_64 + arm64)
#   --arch=ARCH      Build for a specific arch (x86_64 or aarch64)
#

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

OS_NAME=$(uname -s || echo "")
if [[ "$OS_NAME" != "Darwin" ]]; then
  echo "This installer currently supports macOS only." >&2
  exit 2
fi

SKIP_BUILD=false
TARGET_ARCH=""
for arg in "$@"; do
  case "$arg" in
    --skip-build) SKIP_BUILD=true ;;
    --universal) TARGET_ARCH="universal" ;;
    --arch=*) TARGET_ARCH="${arg#--arch=}" ;;
    *) echo "Ignoring unknown option: $arg" ;;
  esac
done

if [[ "$SKIP_BUILD" != true ]]; then
  if [[ -n "$TARGET_ARCH" ]]; then
    TARGET_ARCH="$TARGET_ARCH" bash scripts/macos/build_release.sh
  else
    bash scripts/macos/build_release.sh
  fi
fi

# Build the package
bash scripts/macos/make_pkg.sh

# Find latest generated pkg
PKG_FILE=$(ls -t target/build-it-agent-*.pkg | head -n1)
if [[ -z "${PKG_FILE:-}" ]]; then
  echo "Failed to locate generated pkg under target/." >&2
  exit 3
fi

echo "Installing package: $PKG_FILE"
sudo installer -pkg "$PKG_FILE" -target /

echo "Install complete. Verifying service status..."
USER_UID=$(id -u)
set +e
launchctl print gui/$USER_UID/com.build-it.agent | grep -E 'state|last exit|program' || true
set -e

echo "\nTry the agent at: http://localhost:8765/status and http://localhost:8910/health"
echo "Logs: ~/Library/Logs/build-it-agent.out.log and ~/Library/Logs/build-it-agent.err.log"

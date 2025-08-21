#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
BINARY_PATH="$ROOT_DIR/target/release/build-it-agent"

if [ ! -f "$BINARY_PATH" ]; then
  echo "Release binary not found at $BINARY_PATH"
  echo "Please run: cargo build --release"
  exit 1
fi

# Extract version from Cargo.toml
VERSION=$(grep '^version' "$ROOT_DIR/Cargo.toml" | head -n1 | sed -E 's/version *= *"([^"]+)"/\1/')
if [ -z "$VERSION" ]; then
  VERSION="0.0.0"
fi

# Determine architecture
if command -v dpkg > /dev/null 2>&1; then
  ARCH=$(dpkg --print-architecture)
else
  UNAME_ARCH=$(uname -m)
  case "$UNAME_ARCH" in
    x86_64) ARCH=amd64 ;;
    aarch64|arm64) ARCH=arm64 ;;
    *) ARCH=all ;;
  esac
fi

PKGNAME="build-it-agent_${VERSION}_${ARCH}.deb"
DIST_DIR="$ROOT_DIR/scripts/dist"
BUILD_DIR=$(mktemp -d)
DEB_ROOT="$BUILD_DIR/package_root"

mkdir -p "$DEB_ROOT/DEBIAN" "$DEB_ROOT/usr/bin" "$DEB_ROOT/lib/systemd/system"

# Copy binary
cp "$BINARY_PATH" "$DEB_ROOT/usr/bin/build-it-agent"
chmod 755 "$DEB_ROOT/usr/bin/build-it-agent"

# Copy service file
cp "$ROOT_DIR/scripts/linux/build-it-agent.service" "$DEB_ROOT/lib/systemd/system/build-it-agent.service"
chmod 644 "$DEB_ROOT/lib/systemd/system/build-it-agent.service"

# Create control file
cat > "$DEB_ROOT/DEBIAN/control" <<EOF
Package: build-it-agent
Version: $VERSION
Section: utils
Priority: optional
Architecture: $ARCH
Maintainer: Build It <noreply@example.com>
Depends: libc6 (>= 2.17)
Description: Build It Agent - background service for build-it
 A small agent that runs on startup.
EOF

# post-install script: enable and start service
cat > "$DEB_ROOT/DEBIAN/postinst" <<'EOF'
#!/bin/bash
set -e
if command -v systemctl >/dev/null 2>&1; then
  systemctl daemon-reload || true
  systemctl enable build-it-agent.service >/dev/null 2>&1 || true
  systemctl start build-it-agent.service >/dev/null 2>&1 || true
fi
exit 0
EOF

# pre-removal script: stop and disable service
cat > "$DEB_ROOT/DEBIAN/prerm" <<'EOF'
#!/bin/bash
set -e
if command -v systemctl >/dev/null 2>&1; then
  systemctl stop build-it-agent.service >/dev/null 2>&1 || true
  systemctl disable build-it-agent.service >/dev/null 2>&1 || true
  systemctl daemon-reload || true
fi
exit 0
EOF

chmod 755 "$DEB_ROOT/DEBIAN/postinst" "$DEB_ROOT/DEBIAN/prerm"

# Build .deb
mkdir -p "$DIST_DIR"
if ! command -v dpkg-deb >/dev/null 2>&1; then
  echo "dpkg-deb not found. Please install 'dpkg-deb' (part of dpkg) to build .deb packages."
  exit 1
fi

dpkg-deb --build "$DEB_ROOT" "$DIST_DIR/$PKGNAME"

echo "Built: $DIST_DIR/$PKGNAME"

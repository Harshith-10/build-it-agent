#!/usr/bin/env bash
set -euo pipefail

# Create a signed or unsigned macOS .pkg installer that installs:
# - /usr/local/libexec/build-it-agent/build-it-agent (binary)
# - ~/Library/LaunchAgents/com.build-it.agent.plist (LaunchAgent)
# - An app symlink in /usr/local/bin/build-it-agent for convenience
# Requires: pkgbuild, productbuild (Xcode Command Line Tools), and optional signing identities.

ROOT_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT_DIR"

APP_NAME="build-it-agent"
IDENTIFIER="com.build-it.agent"
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n1)
PKG_DIR="target/pkg"
PAYLOAD_DIR="$PKG_DIR/payload"
SCRIPTS_DIR="$PKG_DIR/scripts"
LAUNCHD_PLIST_TEMPLATE="scripts/macos/templates/com.build-it.agent.plist"
LAUNCHD_PLIST="$PKG_DIR/com.build-it.agent.plist"

rm -rf "$PKG_DIR"
mkdir -p "$PAYLOAD_DIR/usr/local/libexec/$APP_NAME" "$PAYLOAD_DIR/usr/local/bin" "$SCRIPTS_DIR"

# Decide binary path
BIN_PATH="target/release/$APP_NAME"
if [[ ! -x "$BIN_PATH" ]]; then
  echo "Release binary not found at $BIN_PATH. Run scripts/macos/build_release.sh first." >&2
  exit 1
fi

# Copy binary
cp "$BIN_PATH" "$PAYLOAD_DIR/usr/local/libexec/$APP_NAME/$APP_NAME"
chmod 755 "$PAYLOAD_DIR/usr/local/libexec/$APP_NAME/$APP_NAME"

# Create symlink postinstall script for convenience
cat > "$SCRIPTS_DIR/postinstall" << 'EOS'
#!/usr/bin/env bash
set -euo pipefail
BIN_DIR="/usr/local/bin"
APP_DIR="/usr/local/libexec/build-it-agent"
mkdir -p "$BIN_DIR"
ln -sf "$APP_DIR/build-it-agent" "$BIN_DIR/build-it-agent"

# Install LaunchAgent into user's LaunchAgents when installing via GUI:
# Note: product installs as root; to target the current console user, detect it.
CONSOLE_USER=$(stat -f %Su /dev/console)
USER_HOME=$(dscl . -read /Users/$CONSOLE_USER NFSHomeDirectory | awk '{print $2}')
LAUNCH_AGENTS_DIR="$USER_HOME/Library/LaunchAgents"
mkdir -p "$LAUNCH_AGENTS_DIR"
cp "/Library/Application Support/build-it-agent/com.build-it.agent.plist" "$LAUNCH_AGENTS_DIR/com.build-it.agent.plist"
chown "$CONSOLE_USER":"staff" "$LAUNCH_AGENTS_DIR/com.build-it.agent.plist"
chmod 644 "$LAUNCH_AGENTS_DIR/com.build-it.agent.plist"

# Resolve any tildes in log paths to absolute user home so launchd doesn't error with EX_CONFIG
/usr/bin/sed -i '' "s|~/|$USER_HOME/|g" "$LAUNCH_AGENTS_DIR/com.build-it.agent.plist"
mkdir -p "$USER_HOME/Library/Logs"
chown -R "$CONSOLE_USER":"staff" "$USER_HOME/Library/Logs" || true

# Load it for next login and current session
TARGET_UID=$(id -u "$CONSOLE_USER")
launchctl bootout gui/$TARGET_UID/com.build-it.agent 2>/dev/null || true
launchctl bootstrap gui/$TARGET_UID "$LAUNCH_AGENTS_DIR/com.build-it.agent.plist" || true
launchctl enable gui/$TARGET_UID/com.build-it.agent || true
launchctl kickstart -k gui/$TARGET_UID/com.build-it.agent || true
EOS
chmod 755 "$SCRIPTS_DIR/postinstall"

# Preinstall ensures shared support dir exists and copies the template plist there
mkdir -p "$PAYLOAD_DIR/Library/Application Support/build-it-agent"
# Fill the template with install path
sed "s|/usr/local/libexec/BUILD_IT_AGENT/build-it-agent|/usr/local/libexec/$APP_NAME/$APP_NAME|g" "$LAUNCHD_PLIST_TEMPLATE" > "$PAYLOAD_DIR/Library/Application Support/build-it-agent/com.build-it.agent.plist"

# Create component package
COMP_PKG="$PKG_DIR/${APP_NAME}.pkg"
pkgbuild \
  --identifier "$IDENTIFIER" \
  --version "$VERSION" \
  --root "$PAYLOAD_DIR" \
  --scripts "$SCRIPTS_DIR" \
  "$COMP_PKG"

# Build product archive
PRODUCT_PKG="target/${APP_NAME}-${VERSION}.pkg"
# Build a distributable product archive from the component package.
# Note: A custom product definition (Distribution) is not required for a single component pkg;
# passing --product with a system path may fail on newer macOS versions. Omit it here.
productbuild \
  --package "$COMP_PKG" \
  "$PRODUCT_PKG"

echo "Created: $PRODUCT_PKG"

#!/bin/bash
# Installer for build-it-agent on Linux
set -e

AGENT_BINARY="../../target/release/build-it-agent"
INSTALL_PATH="/usr/local/bin/build-it-agent"
SERVICE_PATH="/etc/systemd/system/build-it-agent.service"

if [ "$EUID" -ne 0 ]; then
  echo "Please run as root"
  exit 1
fi

if [ ! -f "$AGENT_BINARY" ]; then
  echo "Agent binary not found at $AGENT_BINARY. Please build it first."
  exit 1
fi

cp "$AGENT_BINARY" "$INSTALL_PATH"
chmod +x "$INSTALL_PATH"

cat <<EOF > "$SERVICE_PATH"
[Unit]
Description=BuildIT Agent
After=network.target

[Service]
Type=simple
ExecStart=$INSTALL_PATH
Restart=on-failure
User=root

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable build-it-agent
systemctl start build-it-agent

echo "build-it-agent installed and enabled to start on boot."

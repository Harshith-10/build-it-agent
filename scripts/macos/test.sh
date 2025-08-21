set -euo pipefail

echo "== PKG files in target =="
ls -l target/*.pkg 2>/dev/null || true

echo "\n== Component pkg(s) in target/pkg =="
ls -l target/pkg/*.pkg 2>/dev/null || true

PKG=$(ls -1 target/*.pkg 2>/dev/null | head -n1 || true)
if [[ -n "$PKG" ]]; then
  echo "\n== Expanding $PKG to /tmp/pkg_expand =="
  rm -rf /tmp/pkg_expand
  pkgutil --expand "$PKG" /tmp/pkg_expand || true
  echo "Contents of expanded product pkg:" 
  ls -R /tmp/pkg_expand || true
fi

echo "\n== Inspect component package payload files via pkgutil --payload-files =="
COMP=$(ls -1 target/pkg/*.pkg 2>/dev/null | head -n1 || true)
if [[ -n "$COMP" ]]; then
  pkgutil --payload-files "$COMP" || true
fi

echo "\n== Check installed locations =="
ls -la /usr/local/libexec/build-it-agent 2>/dev/null || true
ls -la /usr/local/bin/build-it-agent 2>/dev/null || true
ls -la "/Library/Application Support/build-it-agent" 2>/dev/null || true

echo "\n== Check user LaunchAgent (current user) =="
LS_USER=$(stat -f %Su /dev/console)
echo "Console user: $LS_USER"
ls -la /Users/$LS_USER/Library/LaunchAgents/com.build-it.agent.plist 2>/dev/null || true

echo "\n== launchctl status for label =="
launchctl print gui/$(id -u $LS_USER)/com.build-it.agent 2>/dev/null || launchctl list | grep -i build-it || true
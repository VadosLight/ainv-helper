#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
APP_NAME="AInv Helper"
BUILD_DIR="$ROOT/target/release"
APP_DIR="$BUILD_DIR/${APP_NAME}.app"
CONTENTS="$APP_DIR/Contents"
MACOS="$CONTENTS/MacOS"
RESOURCES="$CONTENTS/Resources"
INSTALL_DIR="/Applications/${APP_NAME}.app"

stop_running() {
    if pgrep -x ainv-helper >/dev/null 2>&1; then
        echo "Stopping running AInv Helper..."
        pkill -x ainv-helper || true
        sleep 0.5
    fi
}

build_bundle() {
    echo "Building release binary..."
    cargo build --release --manifest-path "$ROOT/Cargo.toml"

    echo "Creating app bundle..."
    rm -rf "$APP_DIR"
    mkdir -p "$MACOS" "$RESOURCES"

    cp "$BUILD_DIR/ainv-helper" "$MACOS/ainv-helper"
    cp "$ROOT/resources/Info.plist" "$CONTENTS/Info.plist"
    cp "$ROOT/config/default.toml" "$RESOURCES/default-config.toml"

    echo "Built: $APP_DIR"
    echo "Binary: $(ls -la "$MACOS/ainv-helper" | awk '{print $6, $7, $8, $5}')"
}

install_bundle() {
    stop_running
    echo "Installing to $INSTALL_DIR..."
    rm -rf "$INSTALL_DIR"
    cp -R "$APP_DIR" "$INSTALL_DIR"
    echo "Installed: $INSTALL_DIR"
    update_launch_agent "$INSTALL_DIR/Contents/MacOS/ainv-helper"
}

update_launch_agent() {
    local exe="$1"
    local plist="$HOME/Library/LaunchAgents/com.ainv-helper.plist"
    [[ -f "$plist" ]] || return 0

    echo "Updating LaunchAgent path..."
    launchctl unload -w "$plist" 2>/dev/null || true
    cat > "$plist" <<PLIST_EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.ainv-helper</string>
    <key>ProgramArguments</key>
    <array>
        <string>${exe}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <false/>
    <key>ProcessType</key>
    <string>Interactive</string>
</dict>
</plist>
PLIST_EOF
    launchctl load -w "$plist"
}

case "${1:-build}" in
    build)
        build_bundle
        echo ""
        echo "Run:  open \"$APP_DIR\""
        echo "Or:   $0 install"
        ;;
    install)
        build_bundle
        install_bundle
        echo ""
        echo "Run:  open \"$INSTALL_DIR\""
        ;;
    restart)
        build_bundle
        install_bundle
        open "$INSTALL_DIR"
        ;;
    stop)
        stop_running
        echo "Stopped."
        ;;
    *)
        echo "Usage: $0 [build|install|restart|stop]"
        exit 1
        ;;
esac

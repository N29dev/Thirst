#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "This preparation script runs only on macOS." >&2
  exit 1
fi

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TAURI="$ROOT/src-tauri"
mkdir -p "$TAURI/resources"

# The packaged .app requires its own Swift/WKWebView login helper.
# The developer build separately compiles the same source into Cargo OUT_DIR.
swiftc -O "$TAURI/native/macos/ThirstLogin.swift" \
  -framework AppKit -framework WebKit -framework Security \
  -o "$TAURI/resources/ThirstLogin"
chmod 755 "$TAURI/resources/ThirstLogin"

# Produce a native macOS icon from the existing Thirst artwork. Generated
# icons are build outputs, not source images that need to be committed.
ICON_BASE="$(mktemp -d "${TMPDIR:-/tmp}/thirst-icons.XXXXXX")"
ICONSET="$ICON_BASE/Thirst.iconset"
mkdir -p "$ICONSET"
trap 'rm -rf "$ICON_BASE"' EXIT
ICON="$TAURI/icons/icon.png"
for spec in "16:icon_16x16.png" "32:icon_16x16@2x.png" \
            "32:icon_32x32.png" "64:icon_32x32@2x.png" \
            "128:icon_128x128.png" "256:icon_128x128@2x.png" \
            "256:icon_256x256.png" "512:icon_256x256@2x.png" \
            "512:icon_512x512.png" "1024:icon_512x512@2x.png"; do
  size="${spec%%:*}"
  filename="${spec#*:}"
  sips -s format png -z "$size" "$size" "$ICON" --out "$ICONSET/$filename" >/dev/null
done
iconutil -c icns "$ICONSET" -o "$TAURI/icons/icon.icns"
echo "Prepared native Thirst macOS helper and icon."

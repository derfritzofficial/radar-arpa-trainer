#!/bin/bash
# Baut "Radar ARPA Trainer.app" in dist/ – startet per Doppelklick ohne Terminalfenster.
#
#   ./scripts/build-macos-app.sh            # nur bauen
#   ./scripts/build-macos-app.sh --install  # zusätzlich nach /Applications kopieren
set -euo pipefail
cd "$(dirname "$0")/.."

NAME="Radar ARPA Trainer"
BIN="radar-arpa-trainer"
VERSION="$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)"

cargo build --release

APP="dist/${NAME}.app"
rm -rf "dist/${NAME}.app"
mkdir -p "${APP}/Contents/MacOS" "${APP}/Contents/Resources"
cp "target/release/${BIN}" "${APP}/Contents/MacOS/${BIN}"
cp assets/icon.icns "${APP}/Contents/Resources/AppIcon.icns"

cat > "${APP}/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>${NAME}</string>
  <key>CFBundleDisplayName</key><string>${NAME}</string>
  <key>CFBundleIdentifier</key><string>de.training.radar-arpa-trainer</string>
  <key>CFBundleExecutable</key><string>${BIN}</string>
  <key>CFBundleIconFile</key><string>AppIcon</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>${VERSION}</string>
  <key>CFBundleVersion</key><string>${VERSION}</string>
  <key>CFBundleDevelopmentRegion</key><string>de</string>
  <key>CFBundleLocalizations</key><array><string>de</string><string>en</string></array>
  <key>LSApplicationCategoryType</key><string>public.app-category.education</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
PLIST

# Ad-hoc-Signatur (nötig auf Apple Silicon, keine Entwickler-ID erforderlich)
codesign --force --deep --sign - "${APP}" >/dev/null 2>&1 || true

# Finder/Dock auf das neue Icon hinweisen (sonst zeigt macOS ggf. noch das alte aus dem Cache)
touch "${APP}"
LSREG=/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister
[[ -x "$LSREG" ]] && "$LSREG" -f "${APP}" >/dev/null 2>&1 || true

echo "Fertig: ${APP}"

if [[ "${1:-}" == "--install" ]]; then
  rm -rf "/Applications/${NAME}.app"
  cp -R "${APP}" /Applications/
  echo "Installiert: /Applications/${NAME}.app"
fi

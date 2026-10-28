#!/bin/bash
set -e

echo "==> Building Ruster (Release)..."
cargo build --release

echo "==> Packaging Ruster.app bundle..."
APP_DIR="target/release/Ruster.app"
mkdir -p "$APP_DIR/Contents/MacOS"
mkdir -p "$APP_DIR/Contents/Resources"

cp target/release/ruster "$APP_DIR/Contents/MacOS/ruster"
cp assets/AppIcon.icns "$APP_DIR/Contents/Resources/AppIcon.icns"
chmod +x "$APP_DIR/Contents/MacOS/ruster"

VERSION=$(grep -m1 '^version =' Cargo.toml | cut -d '"' -f2)
VERSION=${VERSION:-0.3.8}

cat << EOF > "$APP_DIR/Contents/Info.plist"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>ruster</string>
    <key>CFBundleIdentifier</key>
    <string>com.ruster.app</string>
    <key>CFBundleName</key>
    <string>Ruster</string>
    <key>CFBundleDisplayName</key>
    <string>Ruster</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>LSMinimumSystemVersion</key>
    <string>12.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

if command -v codesign >/dev/null 2>&1; then
    echo "==> Ad-hoc signing Ruster.app..."
    codesign --force --deep --sign - "$APP_DIR"
fi

echo "==> Creating Ruster.zip archive..."
(cd target/release && rm -f Ruster.zip && zip -rq Ruster.zip Ruster.app)

echo "==> Done! Application is ready at: target/release/Ruster.app and target/release/Ruster.zip"

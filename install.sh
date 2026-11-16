#!/usr/bin/env bash

# Exit immediately if a command exits with a non-zero status
set -e

echo "==> Pulling latest changes from git..."
git pull

echo "==> Building Rust project (--release)..."
cargo build --release

# Set destination directory
DEST_DIR="$HOME/Clipy-the-dipy"

echo "==> Creating destination directory: $DEST_DIR"
mkdir -p "$DEST_DIR"


echo "==> Copying binaries to $DEST_DIR..."
# Cargo places compiled binaries in target/release/. 
# Depending on your binary target names in Cargo.toml, adjust the filenames below if needed:
cp ~/Clipboard/target/release/Clipboard-Manager "$DEST_DIR/" || cp target/release/clipboard-manager "$DEST_DIR/" 2>/dev/null || true
cp target/release/ClipboardWriter "$DEST_DIR/" || cp target/release/clipboard-writer "$DEST_DIR/" 2>/dev/null || true

$DEST_DIR/ClipboardWriter &!

echo "==> Successfully built and deployed binaries to $DEST_DIR!"
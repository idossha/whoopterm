#!/bin/bash

# Build script for Homebrew package
# This script builds the macOS binary and prepares it for release

set -e

BINARY_NAME=whoopterm
BUILD_DIR=target/release

echo "Building $BINARY_NAME for macOS..."

# Build for both architectures
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin

echo "Copying binaries to release directory..."
mkdir -p release

# Copy Intel binary
cp $BUILD_DIR/x86_64-apple-darwin/$BINARY_NAME release/$BINARY_NAME-macos

# Copy ARM binary
cp $BUILD_DIR/aarch64-apple-darwin/$BINARY_NAME release/$BINARY_NAME-macos-arm64

echo "Creating universal binary..."
lipo -create release/$BINARY_NAME-macos release/$BINARY_NAME-macos-arm64 -output release/$BINARY_NAME-macos-universal

# Clean up individual binaries
rm release/$BINARY_NAME-macos release/$BINARY_NAME-macos-arm64

# Rename universal binary to standard name
mv release/$BINARY_NAME-macos-universal release/$BINARY_NAME-macos

echo "Build complete! Binary available at: release/$BINARY_NAME-macos"
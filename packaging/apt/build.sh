#!/bin/bash
set -e

VERSION="1.0.0"
ARCH="amd64"
PKG_NAME="whoop-cli_${VERSION}_${ARCH}"

# Create directory structure
mkdir -p ${PKG_NAME}/DEBIAN
mkdir -p ${PKG_NAME}/usr/bin
mkdir -p ${PKG_NAME}/usr/share/doc/whoop-cli

# Copy binary
cp ../../target/release/whoop ${PKG_NAME}/usr/bin/

# Create control file
cat > ${PKG_NAME}/DEBIAN/control << EOF
Package: whoop-cli
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: ${ARCH}
Depends: libc6
Maintainer: idossha <idossha@example.com>
Description: WHOOP fitness dashboard for the terminal
 whoop-cli provides a beautiful terminal interface for viewing
 your WHOOP fitness data including recovery, sleep, and workouts.
EOF

# Copy documentation
cp ../../README.md ${PKG_NAME}/usr/share/doc/whoop-cli/
cp ../../LICENSE ${PKG_NAME}/usr/share/doc/whoop-cli/

# Build package
dpkg-deb --build ${PKG_NAME}

# Cleanup
rm -rf ${PKG_NAME}

echo "Package built: ${PKG_NAME}.deb"

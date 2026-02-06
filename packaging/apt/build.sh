#!/bin/bash
set -e

VERSION="1.0.0"
ARCH="amd64"
PKG_NAME="whoopterm_${VERSION}_${ARCH}"

# Copy binary
cp ../../target/release/whoopterm ${PKG_NAME}/usr/bin/

# Create control file
cat > ${PKG_NAME}/DEBIAN/control << EOF
Package: whoopterm
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: ${ARCH}
Depends: libc6
Maintainer: idossha <idossha@example.com>
Description: WHOOP fitness dashboard for the terminal
 whoopterm provides a beautiful terminal interface for viewing
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

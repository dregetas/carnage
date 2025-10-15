#!/bin/bash

# Build RPM for rust-dnf

set -e

# Variables
PACKAGE_NAME="rust-dnf"
VERSION="0.1.0"
BUILD_DIR="rpm-build"

# Clean previous builds
rm -rf $BUILD_DIR/SOURCES/*.tar.gz
rm -rf $BUILD_DIR/BUILD/*
rm -rf $BUILD_DIR/BUILDROOT/*

# Create source tarball
echo "Creating source tarball..."
tar --exclude='./target' --exclude='./rpm-build' --exclude='./.git' \
    -czf $BUILD_DIR/SOURCES/${PACKAGE_NAME}-${VERSION}.tar.gz .

# Build RPM
echo "Building RPM..."
rpmbuild --define "_topdir $(pwd)/$BUILD_DIR" \
         --define "_version $VERSION" \
         -ba $BUILD_DIR/SPECS/rust-dnf.spec

echo "RPM built successfully!"
echo "RPMS can be found in: $BUILD_DIR/RPMS/"
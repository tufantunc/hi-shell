#!/bin/bash

# hi-shell - Installation Script
# This script detects your OS and architecture, downloads the latest binary from GitHub, 
# and installs it to /usr/local/bin.

set -e

REPO="tufantunc/hi-shell"
BINARY_NAME="hi-shell"
INSTALL_PATH="/usr/local/bin/hi-shell"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}🐚 hi-shell Installation${NC}"

# Detect OS
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
case "${OS}" in
    linux*)     TARGET_OS="unknown-linux-gnu" ;;
    darwin*)    TARGET_OS="apple-darwin" ;;
    *)          echo -e "${RED}Error: Unsupported OS: ${OS}${NC}"; exit 1 ;;
esac

# Detect Architecture
ARCH="$(uname -m)"
case "${ARCH}" in
    x86_64)     TARGET_ARCH="x86_64" ;;
    arm64|aarch64) TARGET_ARCH="aarch64" ;;
    *)          echo -e "${RED}Error: Unsupported architecture: ${ARCH}${NC}"; exit 1 ;;
esac

TARGET="${TARGET_ARCH}-${TARGET_OS}"
echo -e "Detected: ${OS} (${ARCH})"

# Fetch latest version from GitHub
echo -e "Fetching latest version info..."
LATEST_TAG=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST_TAG" ]; then
    echo -e "${RED}Error: Could not find latest release.${NC}"
    exit 1
fi

echo -e "Latest version: ${LATEST_TAG}"

# Construct download URL
# Example: hi-shell-aarch64-apple-darwin.tar.gz
ASSET_NAME="${BINARY_NAME}-${TARGET}.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST_TAG}/${ASSET_NAME}"

# Download and Extract
TMP_DIR=$(mktemp -d)
echo -e "Downloading ${ASSET_NAME}..."
curl -L -o "${TMP_DIR}/${ASSET_NAME}" "${DOWNLOAD_URL}"

echo -e "Extracting..."
tar -xzf "${TMP_DIR}/${ASSET_NAME}" -C "${TMP_DIR}"

# Install
echo -e "Installing to ${INSTALL_PATH} (may require sudo)..."
sudo mv "${TMP_DIR}/${BINARY_NAME}" "${INSTALL_PATH}"
sudo chmod +x "${INSTALL_PATH}"

# Cleanup
rm -rf "${TMP_DIR}"

echo -e "\n${GREEN}✔ hi-shell installed successfully!${NC}"
echo -e "Type ${BLUE}hi-shell --init${NC} to get started."

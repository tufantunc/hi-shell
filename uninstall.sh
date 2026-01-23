#!/bin/bash

# hi-shell - Uninstallation Script
# Removes the binary from /usr/local/bin and optionally cleans up configuration data.

set -e

BINARY_NAME="hi-shell"
INSTALL_PATH="/usr/local/bin/hi-shell"
CONFIG_DIR="${HOME}/.config/hi-shell"

# Colors for output
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${YELLOW}🐚 hi-shell Uninstallation${NC}"

if [ ! -f "$INSTALL_PATH" ]; then
    echo -e "hi-shell is not installed at ${INSTALL_PATH}."
else
    echo -e "Removing ${INSTALL_PATH} (may require sudo)..."
    sudo rm "${INSTALL_PATH}"
    echo -e "Binary removed."
fi

if [ -d "$CONFIG_DIR" ]; then
    echo -e "\nConfiguration data found at: ${CONFIG_DIR}"
    read -p "Do you want to remove configuration data and logs? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        rm -rf "$CONFIG_DIR"
        echo -e "Configuration data removed."
    fi
fi

echo -e "\n${RED}✔ hi-shell has been uninstalled.${NC}"

#!/usr/bin/env bash

set -e

# Colors
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

BINARY_NAME="systemdgenerator-v1.0"
SYMLINK_NAME="systemdgenerator"
RELEASE_URL="https://github.com/hylmithecoder/systemd-generator/releases/download/v1.0/systemdfilegenerator"

echo -e "${CYAN}====================================================${NC}"
echo -e "${CYAN} Installing ${YELLOW}${BINARY_NAME}${NC}"
echo -e "${CYAN}====================================================${NC}"

TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

DOWNLOAD_SUCCESS=false

echo -e "${GREEN}--> Downloading static prebuilt binary from GitHub Release (v1.0)...${NC}"
if command -v curl &> /dev/null; then
    if curl -fsSL "$RELEASE_URL" -o "$TMP_DIR/$BINARY_NAME"; then
        DOWNLOAD_SUCCESS=true
    fi
elif command -v wget &> /dev/null; then
    if wget -qO "$TMP_DIR/$BINARY_NAME" "$RELEASE_URL"; then
        DOWNLOAD_SUCCESS=true
    fi
fi

if [ "$DOWNLOAD_SUCCESS" = false ]; then
    echo -e "${YELLOW}--> Download from GitHub release v1.0 not yet available or failed.${NC}"
    if command -v cargo &> /dev/null; then
        echo -e "${GREEN}--> Compiling static binary locally from source with Cargo (musl)...${NC}"
        rustup target add x86_64-unknown-linux-musl 2>/dev/null || true
        cargo build --release --target x86_64-unknown-linux-musl || cargo build --release
        
        if [ -f "target/x86_64-unknown-linux-musl/release/systemdfilegenerator" ]; then
            cp target/x86_64-unknown-linux-musl/release/systemdfilegenerator "$TMP_DIR/$BINARY_NAME"
        else
            cp target/release/systemdfilegenerator "$TMP_DIR/$BINARY_NAME"
        fi
    else
        echo -e "${RED}Error: Could not download release binary and Cargo is not installed.${NC}"
        exit 1
    fi
fi

chmod +x "$TMP_DIR/$BINARY_NAME"

INSTALL_DIR="/usr/local/bin"

if [ -w "$INSTALL_DIR" ]; then
    cp "$TMP_DIR/$BINARY_NAME" "${INSTALL_DIR}/${BINARY_NAME}"
    ln -sf "${INSTALL_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/${SYMLINK_NAME}"
else
    if sudo -n true 2>/dev/null || sudo cp "$TMP_DIR/$BINARY_NAME" "${INSTALL_DIR}/${BINARY_NAME}" 2>/dev/null; then
        sudo ln -sf "${INSTALL_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/${SYMLINK_NAME}"
    else
        echo -e "${YELLOW}Notice: Cannot write to /usr/local/bin directly. Installing to ~/.local/bin...${NC}"
        INSTALL_DIR="$HOME/.local/bin"
        mkdir -p "$INSTALL_DIR"
        cp "$TMP_DIR/$BINARY_NAME" "${INSTALL_DIR}/${BINARY_NAME}"
        ln -sf "${INSTALL_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/${SYMLINK_NAME}"
    fi
fi

echo -e "${GREEN}====================================================${NC}"
echo -e "${GREEN} SUCCESS! ${BINARY_NAME} installed successfully.${NC}"
echo -e "${GREEN} Executables installed to:${NC}"
echo -e "${CYAN}   - ${INSTALL_DIR}/${BINARY_NAME}${NC}"
echo -e "${CYAN}   - ${INSTALL_DIR}/${SYMLINK_NAME}${NC}"
echo -e "${GREEN}====================================================${NC}"
echo -e "Run '${YELLOW}${BINARY_NAME}${NC}' or '${YELLOW}${SYMLINK_NAME}${NC}' in your terminal."

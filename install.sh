#!/usr/bin/env bash

set -e

# Colors
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

BINARY_NAME="systemdgenerator-v1.0"
INSTALL_DIR="/usr/local/bin"

echo -e "${CYAN}====================================================${NC}"
echo -e "${CYAN} Building & Installing ${YELLOW}${BINARY_NAME}${NC}"
echo -e "${CYAN}====================================================${NC}"

# Check cargo installation
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: 'cargo' is not installed or not in PATH.${NC}"
    exit 1
fi

echo -e "${GREEN}--> Building release binary with Cargo...${NC}"
cargo build --release

SOURCE_BIN="target/release/systemdfilegenerator"

if [ ! -f "$SOURCE_BIN" ]; then
    echo -e "${RED}Error: Compiled binary not found at ${SOURCE_BIN}${NC}"
    exit 1
fi

echo -e "${GREEN}--> Installing binary to ${INSTALL_DIR}/${BINARY_NAME}...${NC}"

# Prompt for sudo if needed
if [ -w "$INSTALL_DIR" ]; then
    cp "$SOURCE_BIN" "${INSTALL_DIR}/${BINARY_NAME}"
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
    ln -sf "${INSTALL_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/systemdgenerator"
else
    echo -e "${YELLOW}Superuser privileges required to write to ${INSTALL_DIR}${NC}"
    sudo cp "$SOURCE_BIN" "${INSTALL_DIR}/${BINARY_NAME}"
    sudo chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
    sudo ln -sf "${INSTALL_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/systemdgenerator"
fi

echo -e "${GREEN}====================================================${NC}"
echo -e "${GREEN} SUCCESS! ${BINARY_NAME} installed successfully.${NC}"
echo -e "${GREEN} Executables available:${NC}"
echo -e "${CYAN}   - ${INSTALL_DIR}/${BINARY_NAME}${NC}"
echo -e "${CYAN}   - ${INSTALL_DIR}/systemdgenerator (symlink)${NC}"
echo -e "${GREEN}====================================================${NC}"
echo -e "Run '${YELLOW}systemdgenerator-v1.0${NC}' or '${YELLOW}systemdgenerator${NC}' in any terminal."

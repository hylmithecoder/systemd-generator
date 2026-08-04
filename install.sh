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
    if curl -fsSL "$RELEASE_URL" -o "$TMP_DIR/$BINARY_NAME" 2>/dev/null; then
        DOWNLOAD_SUCCESS=true
    fi
elif command -v wget &> /dev/null; then
    if wget -qO "$TMP_DIR/$BINARY_NAME" "$RELEASE_URL" 2>/dev/null; then
        DOWNLOAD_SUCCESS=true
    fi
fi

if [ "$DOWNLOAD_SUCCESS" = false ] || [ ! -s "$TMP_DIR/$BINARY_NAME" ]; then
    echo -e "${YELLOW}--> GitHub release v1.0 binary not found online (404/Not Uploaded).${NC}"
    if command -v cargo &> /dev/null; then
        echo -e "${GREEN}--> Compiling static binary locally from source with Cargo (musl)...${NC}"
        rustup target add x86_64-unknown-linux-musl 2>/dev/null || true
        cargo build --release --target x86_64-unknown-linux-musl || cargo build --release
        
        if [ -f "target/x86_64-unknown-linux-musl/release/systemdfilegenerator" ]; then
            cp "target/x86_64-unknown-linux-musl/release/systemdfilegenerator" "$TMP_DIR/$BINARY_NAME"
        elif [ -f "target/release/systemdfilegenerator" ]; then
            cp "target/release/systemdfilegenerator" "$TMP_DIR/$BINARY_NAME"
        else
            echo -e "${RED}Error: Build finished but executable binary was not found.${NC}"
            exit 1
        fi
    else
        echo -e "${RED}Error: Release binary v1.0 not uploaded to GitHub and Cargo is not installed to compile locally.${NC}"
        exit 1
    fi
fi

chmod +x "$TMP_DIR/$BINARY_NAME"

# Get binary size for transparency
if command -v du &> /dev/null; then
    BIN_SIZE=$(du -h "$TMP_DIR/$BINARY_NAME" | cut -f1)
else
    BIN_SIZE=$(ls -lh "$TMP_DIR/$BINARY_NAME" | awk '{print $5}')
fi

# Determine target binary installation directory (/usr/local/bin, /usr/bin, /bin, or ~/.local/bin)
TARGET_DIRS=("/usr/local/bin" "/usr/bin" "/bin" "$HOME/.local/bin")
INSTALL_DIR=""

for dir in "${TARGET_DIRS[@]}"; do
    if [ -d "$dir" ]; then
        if [ -w "$dir" ]; then
            INSTALL_DIR="$dir"
            USE_SUDO=false
            break
        elif command -v sudo &> /dev/null && sudo -n true 2>/dev/null; then
            INSTALL_DIR="$dir"
            USE_SUDO=true
            break
        elif command -v sudo &> /dev/null; then
            INSTALL_DIR="$dir"
            USE_SUDO=true
            break
        fi
    fi
done

if [ -z "$INSTALL_DIR" ]; then
    INSTALL_DIR="$HOME/.local/bin"
    USE_SUDO=false
fi

mkdir -p "$INSTALL_DIR" 2>/dev/null || true

echo -e "${GREEN}--> Installing binary to ${INSTALL_DIR}/${BINARY_NAME}...${NC}"

if [ "$USE_SUDO" = true ]; then
    if sudo cp "$TMP_DIR/$BINARY_NAME" "${INSTALL_DIR}/${BINARY_NAME}" 2>/dev/null; then
        sudo chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
        sudo ln -sf "${BINARY_NAME}" "${INSTALL_DIR}/${SYMLINK_NAME}"
    else
        echo -e "${YELLOW}Notice: sudo operation failed or restricted. Installing to ~/.local/bin...${NC}"
        INSTALL_DIR="$HOME/.local/bin"
        mkdir -p "$INSTALL_DIR"
        cp "$TMP_DIR/$BINARY_NAME" "${INSTALL_DIR}/${BINARY_NAME}"
        chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
        ln -sf "${BINARY_NAME}" "${INSTALL_DIR}/${SYMLINK_NAME}"
    fi
else
    cp "$TMP_DIR/$BINARY_NAME" "${INSTALL_DIR}/${BINARY_NAME}"
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
    ln -sf "${BINARY_NAME}" "${INSTALL_DIR}/${SYMLINK_NAME}"
fi

echo -e "${GREEN}====================================================${NC}"
echo -e "${GREEN} SUCCESS! ${BINARY_NAME} installed successfully.${NC}"
echo -e "${GREEN} Binary Size: ${YELLOW}${BIN_SIZE}${NC}"
echo -e "${GREEN} Executables installed to:${NC}"
echo -e "${CYAN}   - ${INSTALL_DIR}/${BINARY_NAME}${NC}"
echo -e "${CYAN}   - ${INSTALL_DIR}/${SYMLINK_NAME}${NC}"
echo -e "${GREEN}====================================================${NC}"
echo -e "Run '${YELLOW}${BINARY_NAME}${NC}' or '${YELLOW}${SYMLINK_NAME}${NC}' in your terminal."

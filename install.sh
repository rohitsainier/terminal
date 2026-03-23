#!/bin/sh
# BharatLink Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/rohitsainier/terminal/main/install.sh | sh
#
# Downloads the latest pre-built BharatLink binary for your platform
# and installs it to /usr/local/bin (or ~/.local/bin as fallback).

set -e

# ─── Config ───────────────────────────────────────────────────────────
REPO="rohitsainier/terminal"
BINARY_NAME="bharatlink"
INSTALL_DIR="/usr/local/bin"

# ─── Colors ───────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

info()  { printf "${CYAN}[INFO]${NC}  %s\n" "$1"; }
ok()    { printf "${GREEN}[OK]${NC}    %s\n" "$1"; }
warn()  { printf "${YELLOW}[WARN]${NC}  %s\n" "$1"; }
error() { printf "${RED}[ERROR]${NC} %s\n" "$1"; exit 1; }

# ─── Banner ───────────────────────────────────────────────────────────
printf "${BOLD}${CYAN}"
cat << 'BANNER'

  ____  _                     _   _     _       _
 | __ )| |__   __ _ _ __ __ _| |_| |   (_)_ __ | | __
 |  _ \| '_ \ / _` | '__/ _` | __| |   | | '_ \| |/ /
 | |_) | | | | (_| | | | (_| | |_| |___| | | | |   <
 |____/|_| |_|\__,_|_|  \__,_|\__|_____|_|_| |_|_|\_\

  P2P File & Text Sharing — No servers, no accounts.

BANNER
printf "${NC}"

# ─── Detect OS & Architecture ────────────────────────────────────────
detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux)   OS_NAME="linux" ;;
        Darwin)  OS_NAME="macos" ;;
        MINGW*|MSYS*|CYGWIN*) OS_NAME="windows" ;;
        *)       error "Unsupported OS: $OS" ;;
    esac

    case "$ARCH" in
        x86_64|amd64)   ARCH_NAME="x86_64" ;;
        aarch64|arm64)  ARCH_NAME="aarch64" ;;
        *)              error "Unsupported architecture: $ARCH" ;;
    esac

    # Construct target triple
    case "$OS_NAME" in
        linux)   TARGET="${ARCH_NAME}-unknown-linux-gnu" ;;
        macos)   TARGET="${ARCH_NAME}-apple-darwin" ;;
        windows) TARGET="${ARCH_NAME}-pc-windows-msvc" ;;
    esac

    info "Detected platform: ${OS_NAME} ${ARCH_NAME} (${TARGET})"
}

# ─── Get Latest Release ──────────────────────────────────────────────
get_latest_version() {
    info "Fetching latest release..."

    if command -v curl >/dev/null 2>&1; then
        VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed 's/.*"tag_name": *"//;s/".*//')
    elif command -v wget >/dev/null 2>&1; then
        VERSION=$(wget -qO- "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed 's/.*"tag_name": *"//;s/".*//')
    else
        error "Neither curl nor wget found. Please install one of them."
    fi

    if [ -z "$VERSION" ]; then
        error "Could not determine latest version. Check https://github.com/${REPO}/releases"
    fi

    # Strip leading 'v' for download URL if present
    VERSION_NUM="${VERSION#v}"
    info "Latest version: ${VERSION} (${VERSION_NUM})"
}

# ─── Download & Install ──────────────────────────────────────────────
download_and_install() {
    ARCHIVE_NAME="${BINARY_NAME}-${VERSION_NUM}-${TARGET}"

    case "$OS_NAME" in
        windows) ARCHIVE_FILE="${ARCHIVE_NAME}.zip" ;;
        *)       ARCHIVE_FILE="${ARCHIVE_NAME}.tar.gz" ;;
    esac

    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${ARCHIVE_FILE}"

    info "Downloading ${ARCHIVE_FILE}..."

    # Create temp directory
    TMP_DIR=$(mktemp -d)
    trap 'rm -rf "$TMP_DIR"' EXIT

    # Download
    if command -v curl >/dev/null 2>&1; then
        HTTP_CODE=$(curl -fsSL -w "%{http_code}" -o "${TMP_DIR}/${ARCHIVE_FILE}" "$DOWNLOAD_URL" 2>/dev/null) || true
        if [ "$HTTP_CODE" != "200" ] && [ ! -s "${TMP_DIR}/${ARCHIVE_FILE}" ]; then
            error "Download failed (HTTP ${HTTP_CODE}). URL: ${DOWNLOAD_URL}"
        fi
    elif command -v wget >/dev/null 2>&1; then
        wget -q -O "${TMP_DIR}/${ARCHIVE_FILE}" "$DOWNLOAD_URL" || error "Download failed. URL: ${DOWNLOAD_URL}"
    fi

    ok "Downloaded $(du -h "${TMP_DIR}/${ARCHIVE_FILE}" | cut -f1 | xargs)"

    # Extract
    info "Extracting..."
    case "$OS_NAME" in
        windows)
            (cd "$TMP_DIR" && unzip -q "$ARCHIVE_FILE")
            ;;
        *)
            tar -xzf "${TMP_DIR}/${ARCHIVE_FILE}" -C "$TMP_DIR"
            ;;
    esac

    # Find the binary
    BINARY_PATH=$(find "$TMP_DIR" -name "$BINARY_NAME" -type f | head -1)
    if [ -z "$BINARY_PATH" ]; then
        # Try with .exe for Windows
        BINARY_PATH=$(find "$TMP_DIR" -name "${BINARY_NAME}.exe" -type f | head -1)
    fi

    if [ -z "$BINARY_PATH" ]; then
        error "Binary not found in archive"
    fi

    chmod +x "$BINARY_PATH"

    # Install — try /usr/local/bin first, fall back to ~/.local/bin
    if [ -w "$INSTALL_DIR" ] || [ -w "$(dirname "$INSTALL_DIR")" ]; then
        mkdir -p "$INSTALL_DIR"
        mv "$BINARY_PATH" "${INSTALL_DIR}/${BINARY_NAME}"
        ok "Installed to ${INSTALL_DIR}/${BINARY_NAME}"
    elif command -v sudo >/dev/null 2>&1; then
        info "Need sudo to install to ${INSTALL_DIR}"
        sudo mkdir -p "$INSTALL_DIR"
        sudo mv "$BINARY_PATH" "${INSTALL_DIR}/${BINARY_NAME}"
        ok "Installed to ${INSTALL_DIR}/${BINARY_NAME}"
    else
        # Fallback to user directory
        INSTALL_DIR="${HOME}/.local/bin"
        mkdir -p "$INSTALL_DIR"
        mv "$BINARY_PATH" "${INSTALL_DIR}/${BINARY_NAME}"
        ok "Installed to ${INSTALL_DIR}/${BINARY_NAME}"
        warn "Add ${INSTALL_DIR} to your PATH if not already there:"
        printf "  ${BOLD}export PATH=\"\$HOME/.local/bin:\$PATH\"${NC}\n"
    fi
}

# ─── Verify Installation ─────────────────────────────────────────────
verify() {
    if command -v "$BINARY_NAME" >/dev/null 2>&1; then
        INSTALLED_VERSION=$("$BINARY_NAME" --version 2>/dev/null || echo "unknown")
        printf "\n"
        ok "BharatLink installed successfully!"
        printf "\n"
        printf "  ${BOLD}Version:${NC}  %s\n" "$INSTALLED_VERSION"
        printf "  ${BOLD}Binary:${NC}   %s\n" "$(which "$BINARY_NAME")"
        printf "\n"
        printf "  ${CYAN}Get started:${NC}\n"
        printf "    ${BOLD}bharatlink start${NC}           Start the P2P node\n"
        printf "    ${BOLD}bharatlink --help${NC}          Show all commands\n"
        printf "\n"
        printf "  ${CYAN}Quick send:${NC}\n"
        printf "    ${BOLD}bharatlink send file <peer_id> ./photo.jpg${NC}\n"
        printf "    ${BOLD}bharatlink send text <peer_id> \"hello!\"${NC}\n"
        printf "\n"
    else
        warn "Binary installed but not in PATH. Restart your terminal or run:"
        printf "  ${BOLD}export PATH=\"${INSTALL_DIR}:\$PATH\"${NC}\n"
    fi
}

# ─── Main ─────────────────────────────────────────────────────────────
main() {
    detect_platform
    get_latest_version
    download_and_install
    verify
}

main

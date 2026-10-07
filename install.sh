#!/usr/bin/env bash
# ==============================================================================
# aim (AppImage Manager) Installer Script
# https://github.com/yoel3imari/aim
# ==============================================================================

set -euo pipefail

REPO="yoel3imari/aim"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
BOLD='\033[1m'
NC='\033[0m'

info() { printf "${BLUE}${BOLD}[INFO]${NC} %s\n" "$1"; }
success() { printf "${GREEN}${BOLD}[SUCCESS]${NC} %s\n" "$1"; }
warn() { printf "${YELLOW}${BOLD}[WARN]${NC} %s\n" "$1"; }
error() { printf "${RED}${BOLD}[ERROR]${NC} %s\n" "$1" >&2; exit 1; }

# 1. OS check
OS="$(uname -s)"
if [ "$OS" != "Linux" ]; then
    error "aim is only supported on Linux (detected: $OS)."
fi

# 2. Architecture check
ARCH="$(uname -m)"
case "$ARCH" in
    x86_64|amd64)
        TARGET="x86_64-unknown-linux-musl"
        ;;
    aarch64|arm64)
        TARGET="aarch64-unknown-linux-musl"
        ;;
    *)
        error "Unsupported architecture: $ARCH. aim provides pre-built binaries for x86_64 and aarch64."
        ;;
esac

info "Detected platform: Linux ($ARCH) -> target: $TARGET"

# 3. Create install directory
mkdir -p "$INSTALL_DIR"

# 4. Resolve latest release
info "Fetching latest release version from GitHub..."
LATEST_TAG=$(curl -sSL "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | grep '"tag_name":' | head -n 1 | cut -d '"' -f 4 || true)

if [ -z "$LATEST_TAG" ]; then
    warn "Could not query GitHub releases API. Falling back to local cargo installation..."
    if command -v cargo >/dev/null 2>&1; then
        cargo install aim
        success "Installed aim via cargo!"
        exit 0
    else
        error "Unable to resolve release and 'cargo' is not installed."
    fi
fi

info "Latest version is: $LATEST_TAG"
ARCHIVE_NAME="aim-${LATEST_TAG}-${TARGET}.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST_TAG}/${ARCHIVE_NAME}"

# 5. Download and extract
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

info "Downloading $ARCHIVE_NAME..."
if curl -sSL --fail "$DOWNLOAD_URL" -o "$TMP_DIR/$ARCHIVE_NAME"; then
    tar -xzf "$TMP_DIR/$ARCHIVE_NAME" -C "$TMP_DIR"
    cp "$TMP_DIR/aim" "$INSTALL_DIR/aim"
    chmod 755 "$INSTALL_DIR/aim"
else
    warn "Direct release asset not found. Building with cargo..."
    if command -v cargo >/dev/null 2>&1; then
        cargo install --git "https://github.com/${REPO}.git"
    else
        error "Failed to download binary and cargo is not installed."
    fi
fi

# 6. Check PATH
case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *)
        warn "$INSTALL_DIR is not currently in your \$PATH."
        warn "Add this line to your ~/.bashrc or ~/.zshrc:"
        printf "    export PATH=\"%s:\$PATH\"\n" "$INSTALL_DIR"
        ;;
esac

# 7. Check helper dependencies
info "Checking system utilities for optimal desktop integration..."
MISSING_TOOLS=""
command -v unsquashfs >/dev/null 2>&1 || MISSING_TOOLS="$MISSING_TOOLS squashfs-tools"
command -v bwrap >/dev/null 2>&1 || MISSING_TOOLS="$MISSING_TOOLS bubblewrap"
command -v update-desktop-database >/dev/null 2>&1 || MISSING_TOOLS="$MISSING_TOOLS desktop-file-utils"

if [ -n "$MISSING_TOOLS" ]; then
    warn "Recommended system tools missing for full desktop & sandbox support:$MISSING_TOOLS"
    printf "    On Debian/Ubuntu: sudo apt install%s\n" "$MISSING_TOOLS"
    printf "    On Fedora:        sudo dnf install%s\n" "$MISSING_TOOLS"
    printf "    On Arch Linux:    sudo pacman -S%s\n" "$MISSING_TOOLS"
fi

# 8. Setup shell completions if directory exists
if [ -d "$HOME/.local/share/bash-completion/completions" ]; then
    "$INSTALL_DIR/aim" completions bash > "$HOME/.local/share/bash-completion/completions/aim" 2>/dev/null || true
fi
if [ -d "$HOME/.config/fish/completions" ]; then
    "$INSTALL_DIR/aim" completions fish > "$HOME/.config/fish/completions/aim.fish" 2>/dev/null || true
fi

success "aim is installed and ready to use!"
printf "\n${BOLD}Quick Start:${NC}\n"
printf "  aim search blender             # Search 3,000+ AppImages\n"
printf "  aim install kdenlive           # Install and integrate directly\n"
printf "  aim install ~/Downloads/app... # Integrate local downloaded AppImage\n"
printf "  aim list                       # View installed AppImages\n"
printf "  aim --help                     # View all commands\n\n"

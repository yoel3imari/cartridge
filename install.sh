#!/usr/bin/env bash
# ==============================================================================
# Cartridge (cart) Installer Script
# https://github.com/yoel3imari/cartridge
# ==============================================================================

set -euo pipefail

REPO="yoel3imari/cartridge"
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
    error "Cartridge is only supported on Linux (detected: $OS)."
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
        error "Unsupported architecture: $ARCH. Cartridge provides pre-built binaries for x86_64 and aarch64."
        ;;
esac

info "Detected platform: Linux ($ARCH) -> target: $TARGET"

# 3. Create install directory
mkdir -p "$INSTALL_DIR"

# 4. Resolve latest release
info "Fetching latest release version from GitHub..."
LATEST_TAG=$(curl -sSL "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | grep '"tag_name":' | head -n 1 | cut -d '"' -f 4 || true)

if [ -z "$LATEST_TAG" ]; then
    warn "Could not query GitHub releases API. Falling back to git installation via cargo..."
    if command -v cargo >/dev/null 2>&1; then
        cargo install --git "https://github.com/${REPO}.git"
        success "Installed cartridge via cargo!"
        exit 0
    else
        error "Unable to resolve release and 'cargo' is not installed."
    fi
fi

info "Latest version is: $LATEST_TAG"
ARCHIVE_NAME="cartridge-${LATEST_TAG}-${TARGET}.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST_TAG}/${ARCHIVE_NAME}"

# 5. Download and extract
TMP_DIR=$(mktemp -d)
BIN_DIR=""
trap 'rm -rf "$TMP_DIR"' EXIT

info "Downloading $ARCHIVE_NAME..."
if curl -sSL --fail "$DOWNLOAD_URL" -o "$TMP_DIR/$ARCHIVE_NAME"; then
    tar -xzf "$TMP_DIR/$ARCHIVE_NAME" -C "$TMP_DIR"
    if [ -f "$TMP_DIR/cart" ]; then
        BIN_DIR="$TMP_DIR"
    else
        CART_PATH="$(find "$TMP_DIR" -type f -name cart | head -n 1)"
        if [ -n "$CART_PATH" ]; then
            BIN_DIR="$(dirname "$CART_PATH")"
        else
            error "Could not find 'cart' binary inside extracted archive."
        fi
    fi

    cp "$BIN_DIR/cart" "$INSTALL_DIR/cart"
    chmod 755 "$INSTALL_DIR/cart"
    if [ -f "$BIN_DIR/cartridge" ]; then
        cp "$BIN_DIR/cartridge" "$INSTALL_DIR/cartridge"
        chmod 755 "$INSTALL_DIR/cartridge"
    else
        ln -sf "$INSTALL_DIR/cart" "$INSTALL_DIR/cartridge"
    fi
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

# 7. Check helper dependencies across distributions
info "Checking system utilities for optimal desktop integration..."
HAS_FUSE=0
if command -v ldconfig >/dev/null 2>&1 && ldconfig -p 2>/dev/null | grep -q "libfuse\.so\.2"; then
    HAS_FUSE=1
else
    for d in /lib /lib64 /usr/lib /usr/lib64 /usr/lib/*-linux-gnu /usr/local/lib; do
        if [ -f "$d/libfuse.so.2" ]; then
            HAS_FUSE=1
            break
        fi
    done
fi

DISTRO_ID="linux"
DISTRO_VER=""
if [ -f /etc/os-release ]; then
    DISTRO_ID="$(grep -E '^ID=' /etc/os-release | cut -d= -f2 | tr -d '"'\'' ' | tr '[:upper:]' '[:lower:]' || echo "linux")"
    DISTRO_VER="$(grep -E '^VERSION_ID=' /etc/os-release | cut -d= -f2 | tr -d '"'\'' ' || true)"
fi

MISSING_TOOLS=""
command -v unsquashfs >/dev/null 2>&1 || MISSING_TOOLS="$MISSING_TOOLS squashfs"
command -v bwrap >/dev/null 2>&1 || MISSING_TOOLS="$MISSING_TOOLS bubblewrap"
command -v update-desktop-database >/dev/null 2>&1 || MISSING_TOOLS="$MISSING_TOOLS desktop-file-utils"

FUSE_PKG="libfuse2"
if [ "$DISTRO_ID" = "ubuntu" ]; then
    # Ubuntu 24.04+ (Noble Numbat and newer)
    if [ -n "$DISTRO_VER" ] && [ "$(echo "$DISTRO_VER >= 24.0" | bc 2>/dev/null || echo 0)" = "1" ]; then
        FUSE_PKG="libfuse2t64"
    fi
elif [ "$DISTRO_ID" = "fedora" ] || [ "$DISTRO_ID" = "rhel" ] || [ "$DISTRO_ID" = "centos" ]; then
    FUSE_PKG="fuse-libs"
elif [ "$DISTRO_ID" = "arch" ] || [ "$DISTRO_ID" = "manjaro" ] || [ "$DISTRO_ID" = "endeavouros" ]; then
    FUSE_PKG="fuse2"
elif echo "$DISTRO_ID" | grep -qi "suse"; then
    FUSE_PKG="libfuse2"
elif [ "$DISTRO_ID" = "alpine" ]; then
    FUSE_PKG="fuse"
fi

if [ "$HAS_FUSE" -eq 0 ]; then
    MISSING_TOOLS="$MISSING_TOOLS $FUSE_PKG"
fi

if [ -n "$MISSING_TOOLS" ]; then
    warn "Recommended system utilities for optimal performance & sandboxing:$MISSING_TOOLS"
    case "$DISTRO_ID" in
        ubuntu|debian|linuxmint|pop)
            DEB_PKGS=""
            command -v unsquashfs >/dev/null 2>&1 || DEB_PKGS="$DEB_PKGS squashfs-tools"
            command -v bwrap >/dev/null 2>&1 || DEB_PKGS="$DEB_PKGS bubblewrap"
            command -v update-desktop-database >/dev/null 2>&1 || DEB_PKGS="$DEB_PKGS desktop-file-utils"
            [ "$HAS_FUSE" -eq 0 ] && DEB_PKGS="$DEB_PKGS $FUSE_PKG"
            printf "    Run: sudo apt install%s\n" "$DEB_PKGS"
            ;;
        fedora|rhel|centos)
            RPM_PKGS=""
            command -v unsquashfs >/dev/null 2>&1 || RPM_PKGS="$RPM_PKGS squashfs-tools"
            command -v bwrap >/dev/null 2>&1 || RPM_PKGS="$RPM_PKGS bubblewrap"
            command -v update-desktop-database >/dev/null 2>&1 || RPM_PKGS="$RPM_PKGS desktop-file-utils"
            [ "$HAS_FUSE" -eq 0 ] && RPM_PKGS="$RPM_PKGS fuse-libs"
            printf "    Run: sudo dnf install%s\n" "$RPM_PKGS"
            ;;
        arch|manjaro|endeavouros)
            ARCH_PKGS=""
            command -v unsquashfs >/dev/null 2>&1 || ARCH_PKGS="$ARCH_PKGS squashfs-tools"
            command -v bwrap >/dev/null 2>&1 || ARCH_PKGS="$ARCH_PKGS bubblewrap"
            command -v update-desktop-database >/dev/null 2>&1 || ARCH_PKGS="$ARCH_PKGS desktop-file-utils"
            [ "$HAS_FUSE" -eq 0 ] && ARCH_PKGS="$ARCH_PKGS fuse2"
            printf "    Run: sudo pacman -S%s\n" "$ARCH_PKGS"
            ;;
        *suse*)
            ZYP_PKGS=""
            command -v unsquashfs >/dev/null 2>&1 || ZYP_PKGS="$ZYP_PKGS squashfs"
            command -v bwrap >/dev/null 2>&1 || ZYP_PKGS="$ZYP_PKGS bubblewrap"
            command -v update-desktop-database >/dev/null 2>&1 || ZYP_PKGS="$ZYP_PKGS desktop-file-utils"
            [ "$HAS_FUSE" -eq 0 ] && ZYP_PKGS="$ZYP_PKGS libfuse2"
            printf "    Run: sudo zypper install%s\n" "$ZYP_PKGS"
            ;;
        alpine)
            APK_PKGS=""
            command -v unsquashfs >/dev/null 2>&1 || APK_PKGS="$APK_PKGS squashfs-tools"
            command -v bwrap >/dev/null 2>&1 || APK_PKGS="$APK_PKGS bubblewrap"
            command -v update-desktop-database >/dev/null 2>&1 || APK_PKGS="$APK_PKGS desktop-file-utils"
            [ "$HAS_FUSE" -eq 0 ] && APK_PKGS="$APK_PKGS fuse"
            printf "    Run: sudo apk add%s\n" "$APK_PKGS"
            ;;
        *)
            printf "    Install equivalent packages: squashfs-tools, bubblewrap, desktop-file-utils, %s\n" "$FUSE_PKG"
            ;;
    esac
fi

# 8. Setup shell completions if directory exists
if [ -d "$HOME/.local/share/bash-completion/completions" ]; then
    if [ -n "$BIN_DIR" ] && [ -f "$BIN_DIR/completions/cart.bash" ]; then
        cp "$BIN_DIR/completions/cart.bash" "$HOME/.local/share/bash-completion/completions/cart"
    elif [ -x "$INSTALL_DIR/cart" ]; then
        "$INSTALL_DIR/cart" completions bash > "$HOME/.local/share/bash-completion/completions/cart" 2>/dev/null || true
    fi
fi
if [ -d "$HOME/.config/fish/completions" ]; then
    if [ -n "$BIN_DIR" ] && [ -f "$BIN_DIR/completions/cart.fish" ]; then
        cp "$BIN_DIR/completions/cart.fish" "$HOME/.config/fish/completions/cart.fish"
    elif [ -x "$INSTALL_DIR/cart" ]; then
        "$INSTALL_DIR/cart" completions fish > "$HOME/.config/fish/completions/cart.fish" 2>/dev/null || true
    fi
fi
if [ -d "$HOME/.zfunc" ]; then
    if [ -n "$BIN_DIR" ] && [ -f "$BIN_DIR/completions/_cart" ]; then
        cp "$BIN_DIR/completions/_cart" "$HOME/.zfunc/_cart"
    elif [ -x "$INSTALL_DIR/cart" ]; then
        "$INSTALL_DIR/cart" completions zsh > "$HOME/.zfunc/_cart" 2>/dev/null || true
    fi
fi

success "Cartridge (cart) is installed and ready to use!"
printf "\n${BOLD}Quick Start:${NC}\n"
printf "  cart search blender             # Search 3,000+ AppImages\n"
printf "  cart install kdenlive           # Install and integrate directly\n"
printf "  cart install ~/Downloads/app... # Integrate local downloaded AppImage\n"
printf "  cart list                       # View installed AppImages\n"
printf "  cart --help                     # View all commands\n\n"

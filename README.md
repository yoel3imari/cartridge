<div align="center">

# 📼 Cartridge (`cart`)

### The Plug-and-Play Application Cartridge Manager for Linux

[![CI](https://github.com/yoel3imari/cartridge/actions/workflows/ci.yml/badge.svg)](https://github.com/yoel3imari/cartridge/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/yoel3imari/cartridge?color=blue)](https://github.com/yoel3imari/cartridge/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.80%2B-orange.svg)](https://www.rust-lang.org)

**Search, install, integrate, update, sandbox, and manage AppImage application cartridges seamlessly.**

[Installation](#-installation) • [Quick Start](#-quick-start) • [Command Reference](#-command-reference) • [Shell Completions](#-shell-completions) • [How It Works](#-how-it-works)

</div>

---

## 💡 The Cartridge Philosophy

Traditional Linux packages scatter libraries across `/usr/lib`, headers across `/usr/include`, and configuration files throughout the system. 

**Cartridge** treats Linux applications like classic physical game cartridges:
- 🔌 **Plug & Play**: One self-contained binary file. Insert it and it runs immediately.
- 🎨 **Zero Friction**: Deep FreeDesktop integration (app menu launcher, HiDPI icons, terminal symlink) extracted safely without executing untrusted code.
- 🛡️ **Self-Contained & Isolated**: Run untrusted cartridges inside optional Bubblewrap (`bwrap`) sandboxes.
- 🧹 **Clean Ejection**: Uninstalling an application removes all launchers, icons, and symlinks with zero residual clutter.
- 🔄 **Instant Rollback**: Every update backs up the previous working cartridge for one-click instant recovery.

Both `cart` (short, ergonomic) and `cartridge` (descriptive) binaries are provided out of the box.

---

## ✨ Features

- 🔍 **Universal Discovery**: Instant fuzzy search across 3,000+ community applications from AppImageHub and GitHub Releases.
- 📦 **Universal Inputs**: Install by catalog name (`cart install blender`), direct GitHub repo (`cart install neovim/neovim`), direct HTTPS URL, or integrate local downloaded files (`cart install ~/Downloads/app.AppImage`).
- 🖥️ **Native Desktop Integration**:
  - Automatically extracts `.desktop` launchers and 256x256 / SVG icons from the AppImage without running untrusted code.
  - Generates FreeDesktop-compliant launchers and updates the system desktop database (`update-desktop-database`) and GTK icon cache on the fly.
  - Creates terminal-executable symlinks directly in `~/.local/bin/<app>`.
- 🔄 **Atomic Updates & Instant Rollback**: Replace binaries safely with zero downtime. Revert broken upstream releases with `cart rollback <app>`.
- 🛡️ **Built-in Sandboxing**: Run unverified AppImages in an isolated Bubblewrap (`bwrap`) container with read-only root and optional network disabling.
- 🧹 **Zero-Residue Removal**: Cleanly removes binaries, launchers, icons, and symlinks with `cart remove <app>`.
- ⚡ **Zero-Dependency Static Binary**: Written in modern Rust, single binary, boots in <10ms.
- 🧳 **Seamless Migration**: Automatically detects and migrates existing legacy installations from `aim` without data loss.

---

## 🚀 Installation

### 1. One-Line Installer (Recommended)
```bash
curl -fsSL https://raw.githubusercontent.com/yoel3imari/cartridge/main/install.sh | bash
```

### 2. Cargo (From Crates.io or Git)
```bash
cargo install cartridge
# or directly from git:
cargo install --git https://github.com/yoel3imari/cartridge.git
```

### 3. Arch Linux (AUR)
```bash
yay -S cartridge-bin
# or
paru -S cartridge-bin
```

### 4. Pre-Built Standalone Binaries
Download the latest static binary for your architecture from [GitHub Releases](https://github.com/yoel3imari/cartridge/releases):
- `x86_64` (64-bit Intel/AMD): [cartridge-v0.1.0-x86_64-unknown-linux-musl.tar.gz](https://github.com/yoel3imari/cartridge/releases)
- `aarch64` (64-bit ARM / Raspberry Pi): [cartridge-v0.1.0-aarch64-unknown-linux-musl.tar.gz](https://github.com/yoel3imari/cartridge/releases)

---

## ⚡ Quick Start

```bash
# 1. Search for an application across 3,000+ AppImages
cart search kdenlive

# 2. View details before installing
cart info kdenlive

# 3. Install from catalog
cart install kdenlive

# 4. Or install directly from GitHub releases
cart install neovim/neovim

# 5. Or integrate a downloaded AppImage from your disk
cart install ~/Downloads/filmcraft-0.2.1-linux-x86_64.AppImage

# 6. List all managed applications
cart list

# 7. Run safely in an isolated sandbox
cart run --sandbox filmcraft

# 8. Check and apply updates
cart update --all

# 9. Cleanly uninstall when finished
cart remove filmcraft
```

> **Note**: You can also use the full `cartridge` command name interchangeably with `cart`.

---

## 📖 Command Reference

| Command | Description | Example |
| :--- | :--- | :--- |
| `cart search <query>` | Search catalog, GitHub, and installed apps | `cart search blender --limit 5` |
| `cart install <target>` | Install by catalog name, GitHub `owner/repo`, URL, or local file | `cart install neovim/neovim` |
| `cart integrate <path>` | Explicitly integrate a local AppImage file | `cart integrate app.AppImage` |
| `cart list` | List all managed AppImages with versions and sizes | `cart list` |
| `cart info <app>` | View detailed info, license, and file paths | `cart info kdenlive` |
| `cart update [app]` | Update one or all installed AppImages | `cart update --all` |
| `cart rollback <app>` | Roll back an app to its previous backup version | `cart rollback neovim` |
| `cart remove <app>` | Cleanly remove app binary, launcher, and icons | `cart remove kdenlive [--purge]` |
| `cart run <app>` | Run an installed AppImage (with optional isolation) | `cart run --sandbox <app>` |
| `cart clean` | Remove dangling symlinks, broken launchers, and old backups | `cart clean` |
| `cart refresh` | Force refresh the local AppImageHub catalog cache | `cart refresh` |
| `cart completions <shell>` | Generate shell autocomplete scripts | `cart completions zsh` |

---

## 🐚 Shell Completions

Generate shell completions dynamically for your favorite shell:

### Bash
```bash
mkdir -p ~/.local/share/bash-completion/completions
cart completions bash > ~/.local/share/bash-completion/completions/cart
```

### Zsh
```bash
mkdir -p ~/.zfunc
cart completions zsh > ~/.zfunc/_cart
# Add to ~/.zshrc: fpath=(~/.zfunc $fpath); autoload -Uz compinit && compinit
```

### Fish
```bash
mkdir -p ~/.config/fish/completions
cart completions fish > ~/.config/fish/completions/cart.fish
```

---

## 🛠️ How It Works (XDG Standards)

`cartridge` strictly follows the FreeDesktop (XDG) Base Directory specification:

```
~/.local/share/cartridge/
├── apps/
│   └── <app-id>/
│       ├── <app-name>.AppImage      # Active executable cartridge
│       └── <app-name>.AppImage.old  # Retained for instant rollback
└── installed.json                   # Registry tracking versions & hashes

~/.local/bin/
├── cart                             # Primary manager CLI
├── cartridge                        # Manager alias
└── <app-id>                         # Direct launcher symlink in $PATH

~/.local/share/applications/
└── cart-<app-id>.desktop            # FreeDesktop launcher with unquoted TryExec

~/.local/share/icons/hicolor/
└── 256x256/apps/
    └── cart-<app-id>.png            # Extracted application icon
```

---

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details on setting up the development environment, running tests, and opening pull requests.

## 📄 License

Distributed under the **MIT License**. See [LICENSE](LICENSE) for more information.

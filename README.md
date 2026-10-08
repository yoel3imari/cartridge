<div align="center">

# <div style="display: flex; align-items: center; justify-content: center; gap: 0.5rem;"><img src="docs/logo.png" style=" height: 3rem;" /> Cartridge (`cart`)</div>

### The Plug-and-Play Application Cartridge Manager for Linux

[![CI](https://github.com/yoel3imari/cartridge/actions/workflows/ci.yml/badge.svg)](https://github.com/yoel3imari/cartridge/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/yoel3imari/cartridge?color=blue)](https://github.com/yoel3imari/cartridge/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.80%2B-orange.svg)](https://www.rust-lang.org)

**Search, install, integrate, update, sandbox, and manage Linux AppImage cartridges with speed and precision.**

[Installation](#installation) • [Quick Start](#quick-start) • [Command Reference](#command-reference) • [Shell Completions](#shell-completions) • [System Architecture](#system-architecture-and-xdg-standards) • [Roadmap](docs/TODO.md)

</div>

---

## Design Concept

Standard package managers distribute files across `/usr/bin`, `/usr/lib`, and `/usr/share`. This distribution can create dependency conflicts and leave unused files on the system after removal.

**Cartridge** implements the physical cartridge model for Linux software:
- **Self-Contained Execution**: Each application is a single executable file.
- **Safe Desktop Integration**: Extracts desktop menu entries, icons, and shell symlinks without executing untrusted binaries.
- **Process Isolation**: Executes applications inside optional Bubblewrap (`bwrap`) containers with restricted permissions.
- **Zero-Residue Removal**: Deletes binaries, menu entries, icons, and symlinks completely during uninstallation.
- **Instant Rollback**: Automatically preserves the previous version during updates for immediate state recovery.

You can use the short command `cart` or the full command `cartridge`. Both provide identical functionality.

---

## Key Capabilities

- **Fast Application Discovery**: Perform fuzzy search across more than 3,000 applications from AppImageHub and GitHub Releases.
- **Flexible Installation Sources**: Install applications from the catalog (`cart install blender`), GitHub repositories (`cart install neovim/neovim`), direct URLs, or local `.AppImage` files.
- **Native Desktop Integration**:
  - Extracts `.desktop` files and high-resolution icons directly from the SquashFS archive without running untrusted code.
  - Generates standard FreeDesktop desktop entries and updates the system desktop database (`update-desktop-database`) and GTK icon cache.
  - Creates executable symlinks directly in `~/.local/bin/<app>`.
- **Atomic Updates and Rollbacks**: Replaces binaries atomically. Revert to the previous working version at any time with `cart rollback <app>`.
- **Integrated Sandboxing**: Runs unverified applications inside an isolated Bubblewrap (`bwrap`) container with a read-only root filesystem and optional network isolation.
- **Complete Uninstallation**: Removes all application files and integration entries with `cart remove <app>`.
- **Lightweight Native Binary**: Written in Rust. Single binary with zero external runtime dependencies and startup time under 10 ms.
- **Automated Migration**: Automatically detects and migrates legacy data configurations without data loss.

---

## Installation

### 1. Automated Installer (Recommended)
This script downloads the binary to `~/.local/bin`, verifies dependencies, and configures shell completions:
```bash
curl -fsSL https://raw.githubusercontent.com/yoel3imari/cartridge/main/install.sh | bash
```

### 2. Cargo Installation from Git
Install directly from source code with Cargo:
```bash
cargo install --git https://github.com/yoel3imari/cartridge.git
```

Or install from a local repository clone:
```bash
git clone https://github.com/yoel3imari/cartridge.git
cd cartridge
cargo install --path .
```

---

## Quick Start

Execute common operations with the following commands:

```bash
# 1. Search for an application in the catalog
cart search kdenlive

# 2. View application details before installation
cart info kdenlive

# 3. Install an application from the catalog
cart install kdenlive

# 4. Install an application directly from a GitHub repository
cart install neovim/neovim

# 5. Integrate an existing local AppImage file
cart install ~/Downloads/filmcraft-0.2.1-linux-x86_64.AppImage

# 6. List all installed applications
cart list

# 7. Run an application inside an isolated sandbox
cart run --sandbox filmcraft

# 8. Check and apply updates for all applications
cart update --all

# 9. Remove an installed application completely
cart remove filmcraft
```

> **Note**: You can use `cartridge` as an alias for `cart` in all commands.

---

## Command Reference

| Command | Description | Example |
| :--- | :--- | :--- |
| `cart search <query>` | Search the catalog, GitHub, and local installations | `cart search blender --limit 5` |
| `cart install <target>` | Install by catalog name, GitHub `owner/repo`, URL, or local file | `cart install neovim/neovim` |
| `cart integrate <path>` | Integrate an existing local AppImage file | `cart integrate app.AppImage` |
| `cart list` | List installed applications with version and size data | `cart list` |
| `cart info <app>` | Display application metadata, license, and file locations | `cart info kdenlive` |
| `cart update [app]` | Update a specific application or all applications | `cart update --all` |
| `cart rollback <app>` | Revert an application to its previous version | `cart rollback neovim` |
| `cart remove <app>` | Remove application binary, desktop launcher, and icons | `cart remove kdenlive [--purge]` |
| `cart run <app>` | Execute an application with optional sandbox isolation | `cart run --sandbox <app>` |
| `cart clean` | Remove invalid symlinks, broken launchers, and stale files | `cart clean` |
| `cart refresh` | Update the local AppImageHub catalog cache | `cart refresh` |
| `cart completions <shell>` | Generate shell completion scripts | `cart completions zsh` |

---

## Shell Completions

Generate completion scripts for your shell:

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

## System Architecture and XDG Standards

Cartridge complies with the FreeDesktop (XDG) Base Directory specification:

```
~/.local/share/cartridge/
├── apps/
│   └── <app-id>/
│       ├── <app-name>.AppImage      # Active executable cartridge
│       └── <app-name>.AppImage.old  # Backup copy for rollback operations
└── installed.json                   # Local state registry with version and hash data

~/.local/bin/
├── cart                             # Primary manager executable
├── cartridge                        # Manager alias executable
└── <app-id>                         # Symlink to active application executable in $PATH

~/.local/share/applications/
└── cart-<app-id>.desktop            # Desktop entry file with unquoted TryExec path

~/.local/share/icons/hicolor/
└── 256x256/apps/
    └── cart-<app-id>.png            # Extracted application icon
```

---

## Roadmap

See [docs/TODO.md](docs/TODO.md) for planned features and technical specifications, including:
- **`zsync` Delta Updates**: Download only binary differences for large AppImage updates.
- **Ephemeral Execution**: Run applications on demand without persistent integration (`cart try` / `cart run --temp`).
- **Shell Completions and Man Pages**: Extract manual pages and completion files during integration.
- **Granular Sandboxing Profiles**: Configure modular permissions (audio, GPU, network, and storage) per application.

---

## Contributing

Contributions are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) for instructions on repository setup, test execution, and code submission.

---

## License

Distributed under the **MIT License**. See [LICENSE](LICENSE) for full terms.

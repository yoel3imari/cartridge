<div align="center">

# 🎯 aim

### The Modern AppImage CLI Manager for Linux

[![CI](https://github.com/yoel3imari/aim/actions/workflows/ci.yml/badge.svg)](https://github.com/yoel3imari/aim/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/yoel3imari/aim?color=blue)](https://github.com/yoel3imari/aim/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.80%2B-orange.svg)](https://www.rust-lang.org)

**Search, install, integrate, update, sandbox, and manage AppImage applications seamlessly.**

[Installation](#-installation) • [Quick Start](#-quick-start) • [Command Reference](#-command-reference) • [Shell Completions](#-shell-completions) • [How It Works](#-how-it-works)

</div>

---

## ✨ Features

- 🔍 **Global Discovery**: Instant fuzzy search across ~3,000 community applications from AppImageHub and direct GitHub releases.
- 📦 **Universal Inputs**: Install by catalog name (`aim install blender`), direct GitHub repo (`aim install neovim/neovim`), HTTPS URL, or integrate local downloaded files (`aim install ~/Downloads/app.AppImage`).
- 🖥️ **Native Desktop Integration**:
  - Automatically extracts `.desktop` launchers and 256x256 / SVG icons from the AppImage without executing untrusted code.
  - Generates FreeDesktop-compliant launchers and updates the system desktop database (`update-desktop-database`) and GTK icon cache on the fly.
  - Creates terminal-executable symlinks directly in `~/.local/bin/<app>`.
- 🔄 **Atomic Updates & Instant Rollback**: Replace binaries safely with zero downtime. Revert broken upstream releases with `aim rollback <app>`.
- 🛡️ **Built-in Sandboxing**: Run unverified AppImages in an isolated Bubblewrap (`bwrap`) container with read-only root and optional network disabling.
- 🧹 **Zero-Residue Removal**: Cleanly removes binaries, launchers, icons, and symlinks with `aim remove <app>`.
- ⚡ **Zero-Dependency Static Binary**: Written in Rust, single binary, boots in <10ms.

---

## 🚀 Installation

### 1. One-Line Installer (Recommended)
```bash
curl -fsSL https://raw.githubusercontent.com/yoel3imari/aim/main/install.sh | bash
```

### 2. Cargo (From Crates.io or Git)
```bash
cargo install aim
# or directly from git:
cargo install --git https://github.com/yoel3imari/aim.git
```

### 3. Arch Linux (AUR)
```bash
yay -S aim-bin
# or
paru -S aim-bin
```

### 4. Pre-Built Standalone Binaries
Download the latest static binary for your architecture from [GitHub Releases](https://github.com/yoel3imari/aim/releases):
- `x86_64` (64-bit Intel/AMD): [aim-v0.1.0-x86_64-unknown-linux-musl.tar.gz](https://github.com/yoel3imari/aim/releases)
- `aarch64` (64-bit ARM / Raspberry Pi): [aim-v0.1.0-aarch64-unknown-linux-musl.tar.gz](https://github.com/yoel3imari/aim/releases)

---

## ⚡ Quick Start

```bash
# 1. Search for an application
aim search kdenlive

# 2. View details before installing
aim info kdenlive

# 3. Install from catalog
aim install kdenlive

# 4. Or install directly from GitHub releases
aim install neovim/neovim

# 5. Or integrate a downloaded AppImage from your disk
aim install ~/Downloads/filmcraft-0.2.1-linux-x86_64.AppImage

# 6. List all installed applications
aim list

# 7. Run safely in an isolated sandbox
aim run --sandbox filmcraft

# 8. Check and apply updates
aim update --all

# 9. Cleanly uninstall when finished
aim remove filmcraft
```

---

## 📖 Command Reference

| Command | Description | Example |
| :--- | :--- | :--- |
| `aim search <query>` | Search catalog, GitHub, and installed apps | `aim search blender --limit 5` |
| `aim install <target>` | Install by catalog name, GitHub `owner/repo`, URL, or local file | `aim install neovim/neovim` |
| `aim integrate <path>` | Explicitly integrate a local AppImage file | `aim integrate app.AppImage` |
| `aim list` | List all managed AppImages with versions and sizes | `aim list` |
| `aim info <app>` | View detailed info, license, and file paths | `aim info kdenlive` |
| `aim update [app]` | Update one or all installed AppImages | `aim update --all` |
| `aim rollback <app>` | Roll back an app to its previous backup version | `aim rollback neovim` |
| `aim remove <app>` | Cleanly remove app binary, launcher, and icons | `aim remove kdenlive [--purge]` |
| `aim run <app>` | Run an installed AppImage (with optional isolation) | `aim run --sandbox <app>` |
| `aim clean` | Remove dangling symlinks, broken launchers, and old backups | `aim clean` |
| `aim refresh` | Force refresh the local AppImageHub catalog cache | `aim refresh` |
| `aim completions <shell>` | Generate shell autocomplete scripts | `aim completions zsh` |

---

## 🐚 Shell Completions

Generate shell completions dynamically for your favorite shell:

### Bash
```bash
mkdir -p ~/.local/share/bash-completion/completions
aim completions bash > ~/.local/share/bash-completion/completions/aim
```

### Zsh
```bash
mkdir -p ~/.zfunc
aim completions zsh > ~/.zfunc/_aim
# Add to ~/.zshrc: fpath=(~/.zfunc $fpath); autoload -Uz compinit && compinit
```

### Fish
```bash
mkdir -p ~/.config/fish/completions
aim completions fish > ~/.config/fish/completions/aim.fish
```

---

## 🛠️ How It Works (XDG Standards)

`aim` follows the FreeDesktop (XDG) Base Directory specification:

```
~/.local/share/aim/
├── apps/
│   └── <app-id>/
│       ├── <app-name>.AppImage      # Active executable binary
│       └── <app-name>.AppImage.old  # Retained for instant rollback
└── installed.json                   # Registry tracking versions & hashes

~/.local/bin/
└── <app-id>                         # Symlink in $PATH for terminal use

~/.local/share/applications/
└── aim-<app-id>.desktop             # FreeDesktop launcher with unquoted TryExec

~/.local/share/icons/hicolor/
└── 256x256/apps/
    └── aim-<app-id>.png             # Extracted application icon
```

---

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details on setting up the development environment, running tests, and opening pull requests.

## 📄 License

Distributed under the **MIT License**. See [LICENSE](LICENSE) for more information.

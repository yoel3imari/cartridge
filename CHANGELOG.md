# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-10-07

### Added
- **Global Catalog Search**: Integrated with AppImageHub (~3,000 community applications) with automatic background caching and Skim fuzzy matching.
- **Direct GitHub Releases Support**: Resolve and install AppImages directly using `owner/repo` identifiers (e.g. `aim install neovim/neovim`).
- **Direct URL & Local File Support**: Install from direct HTTPS URLs or integrate existing local `.AppImage` files.
- **Native FreeDesktop Integration**:
  - Non-destructive SquashFS inspection via `unsquashfs -offset` without executing untrusted code.
  - Automatic extraction and installation of 256x256 and scalable SVG icons into `~/.local/share/icons/hicolor/`.
  - Automatic generation and mutation of `.desktop` entry files complying strictly with the FreeDesktop specification.
  - Automatic `$PATH` symlink creation in `~/.local/bin/`.
  - Background database refresh via `update-desktop-database` and `gtk-update-icon-cache`.
- **Application State Tracking**: JSON registry in `~/.local/share/aim/installed.json` tracking installed apps, sizes, SHA-256 hashes, and paths.
- **Atomic Updates & Instant Rollback**: Replace binaries atomically while retaining previous versions for instant one-command rollback (`aim rollback <app>`).
- **Bubblewrap Sandboxing**: Optional isolated container execution via `aim run --sandbox <app>` with read-only root, isolated tmpfs, and optional network disabling.
- **Clean Uninstallation**: Zero-residue removal of binaries, symlinks, desktop files, and icons (`aim remove <app>`).
- **Shell Autocompletions**: Built-in autocompletion generator for `bash`, `zsh`, `fish`, `elvish`, and `powershell` (`aim completions <shell>`).
- **Installer Script & Cross-Compilation**: `install.sh` one-liner installer and GitHub Actions release matrix for `x86_64` and `aarch64` (GNU and static MUSL).

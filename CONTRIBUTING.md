# Contributing to Cartridge

Thank you for your interest in contributing to **`Cartridge` (`cart`)**!

We welcome pull requests, bug reports, documentation improvements, and feature suggestions.

---

## Development Setup

### Prerequisites
- **Rust Toolchain**: Rust 1.80+ (recommended: latest stable `rustup update`)
- **System Tools** (for full testing on Linux):
  ```bash
  # Debian/Ubuntu
  sudo apt install squashfs-tools bubblewrap desktop-file-utils build-essential

  # Fedora
  sudo dnf install squashfs-tools bubblewrap desktop-file-utils

  # Arch Linux
  sudo pacman -S squashfs-tools bubblewrap desktop-file-utils
  ```

---

## Building & Testing

```bash
# Clone the repository
git clone https://github.com/yoel3imari/cartridge.git
cd cartridge

# Build debug binaries (cart & cartridge)
cargo build

# Run unit and integration tests
cargo test

# Run linter
cargo clippy --all-targets -- -D warnings

# Check code formatting
cargo fmt --check
```

---

## Code Quality Standards

Before opening a pull request, please make sure:
1. `cargo test` passes cleanly with all tests green.
2. `cargo clippy --all-targets -- -D warnings` emits zero warnings.
3. `cargo fmt --check` passes without differences (run `cargo fmt` to apply).

---

## Commit Guidelines

We follow [Conventional Commits](https://www.conventionalcommits.org/):
- `feat:` New features or capabilities
- `fix:` Bug fixes
- `docs:` Documentation improvements
- `refactor:` Code refactoring without behavioral changes
- `test:` Adding or updating tests
- `chore:` Release, build system, or dependency updates

---

## License

By contributing to Cartridge, you agree that your contributions will be licensed under the project's [MIT License](LICENSE).

# Contributing to aim

Thank you for your interest in contributing to **`aim`**!

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
git clone https://github.com/yoel3imari/aim.git
cd aim

# Build debug binary
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
1. `cargo test` passes cleanly.
2. `cargo clippy --all-targets -- -D warnings` emits zero warnings.
3. `cargo fmt --check` passes without differences (run `cargo fmt` to apply).

---

## Commit Guidelines

We follow [Conventional Commits](https://www.conventionalcommits.org/):
- `feat: add support for custom repository catalogs`
- `fix: ensure TryExec is never quoted in desktop entries`
- `docs: update installation instructions in README`
- `test: add unit test for arch scoring`

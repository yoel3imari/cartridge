# Cartridge (`cart`) — Future Feature Roadmap & TODO

This document tracks upcoming features and architectural enhancements for future versions of **Cartridge**, focusing on high-performance updates, extended system integration, and flexible sandboxing for application cartridges.

---

## 📋 Summary Roadmap

- [ ] **1. Delta Updates (`zsync` Support)**
- [ ] **2. Ephemeral Execution (`cart try` / `cart run --temp`)**
- [ ] **3. Extended System Integration (Man Pages & Shell Completions)**
- [ ] **4. Advanced Sandboxing & Permission Profiles**
- [ ] **5. Static Binaries & Alternative Portable Formats**
- [ ] **6. Declarative Recipes & Checksum Pinning**

---

## 🚀 Detailed Feature Specifications & Tasks

### 1. Delta Updates (`zsync` Support)
> **Goal:** Significantly reduce update times and bandwidth usage for large AppImages (e.g., Blender, Krita, LibreOffice) by downloading only changed binary chunks instead of multi-hundred-megabyte files.

- [ ] **Inspect AppImage Update Information**
  - Read `.upd_info` embedded in the ELF header / AppImage Type 2 metadata (`zsync|http://...` or `gh-releases-zsync|...`).
  - Add update info parser to [`src/extractor/squashfs.rs`](file:///home/xozev/projects/cartridge/src/extractor/squashfs.rs).
- [ ] **Zsync Protocol / Chunk Fetcher**
  - Download and parse the remote `.zsync` control file (header, block sizes, checksums).
  - Use HTTP `Range` requests via `reqwest` to fetch only missing blocks and patch the existing cartridge into the new release.
  - Implement in [`src/downloader/zsync.rs`](file:///home/xozev/projects/cartridge/src/downloader/) with fallback to full download if delta patching fails.
- [ ] **CLI & Manager Integration**
  - Add delta update statistics to `cart update` (e.g., "Updated 850 MB app using 42 MB delta download").

---

### 2. Ephemeral / Try Execution (`cart try` or `cart run --temp`)
> **Goal:** Allow users to run an application directly from a catalog name, GitHub repo, or remote URL without permanent system integration or lingering desktop files.

- [ ] **CLI Syntax & Arguments**
  - Implement `cart try <target>` and `cart run --temp <target>` in [`src/cli.rs`](file:///home/xozev/projects/cartridge/src/cli.rs).
  - Support forwarding arguments: `cart try neovim/neovim -- file.txt`.
- [ ] **Ephemeral Lifecycle Management**
  - Download into XDG runtime / cache directory (`$XDG_CACHE_HOME/cartridge/ephemeral/<app-id>`).
  - Bypass `.desktop` file and icon registration in [`src/integrator/`](file:///home/xozev/projects/cartridge/src/integrator/).
  - Execute directly or inside a Bubblewrap sandbox (`--sandbox`).
  - Provide prompt or flag to promote an ephemeral run into a permanent install: `cart keep <app-id>`.

---

### 3. Extended System Integration (Man Pages & Shell Completions)
> **Goal:** Extract and register CLI artifacts packaged inside AppImages so command-line cartridges feel like native system packages.

- [ ] **SquashFS Payload Discovery**
  - Detect `usr/share/man/` (sections 1 through 8) inside the SquashFS filesystem.
  - Detect shell completion scripts in `usr/share/bash-completion/`, `usr/share/zsh/`, and `usr/share/fish/`.
- [ ] **FreeDesktop & Shell Registration**
  - Symlink or copy man pages into `~/.local/share/man/man*/`.
  - Link completions into `~/.local/share/bash-completion/completions/`, `~/.local/share/zsh/site-functions/`, and `~/.local/share/fish/vendor_completions.d/`.
  - Trigger `mandb -u` (if present) to refresh man indices.
- [ ] **State & Cleanup**
  - Record installed man pages and completions in [`src/manager/state.rs`](file:///home/xozev/projects/cartridge/src/manager/state.rs).
  - Ensure `cart remove` completely deletes these artifacts without leaving dead files.

---

### 4. Advanced Sandboxing & Permission Profiles
> **Goal:** Expand Cartridge's Bubblewrap (`bwrap`) isolation into modular, fine-grained permission controls.

- [ ] **Modular Sandbox Flags**
  - `--allow-gpu` / `--no-gpu` (DRI / Vulkan / VAAPI access).
  - `--allow-sound` / `--no-sound` (PipeWire / PulseAudio sockets).
  - `--allow-wayland` / `--allow-x11` display server isolation.
  - `--allow-network` / `--offline`.
  - `--bind-dir <host_path>:<container_path>` for custom filesystem mounts.
- [ ] **App Sandbox Presets / Profiles**
  - Allow storing per-app sandbox preferences in [`cartridge.toml`](file:///home/xozev/projects/cartridge/src/manager/state.rs) (e.g. always launch Spotify with audio enabled, but without arbitrary home directory read access).
  - Isolated XDG data/config paths per cartridge (`~/.local/share/cartridge/sandboxes/<app>/`).

---

### 5. Static Binaries & Alternative Portable Formats
> **Goal:** Support non-AppImage portable formats (static CLI binaries, FlatImage, RunImage) under the same unified Cartridge interface.

- [ ] **Single Static Binary Detection & Integration**
  - Support installing standalone static ELF binaries (e.g., `ripgrep`, `jq`, `bat`).
  - Automatically install executable to `~/.local/bin/<name>` without SquashFS extraction requirement.
- [ ] **Support for FlatImage & RunImage Containers**
  - Detect FlatImage/RunImage containers and apply appropriate execution flags and desktop integration.

---

### 6. Declarative Recipes & Checksum Pinning
> **Goal:** Support reproducible, curated installations with cryptographic verification for software not hosted on AppImageHub or GitHub Releases.

- [ ] **Recipe Specification (TOML)**
  - Define local or remote recipe schema: source URL, SHA-256 hash, desktop name, category, default sandbox profile, and launch arguments.
- [ ] **Repository / Feed Support**
  - Allow adding custom recipe registries/indexes: `cart repo add <url>`.
  - Verify SHA-256 checksums automatically before installation or execution.

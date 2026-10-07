use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::util::xdg::XdgPaths;

pub struct SymlinkManager;

impl SymlinkManager {
    /// Ensure binary is executable and create symlink in ~/.local/bin/
    pub fn create_bin_symlink(
        xdg: &XdgPaths,
        app_id: &str,
        target_binary: &Path,
    ) -> Result<PathBuf> {
        // Ensure binary has executable permissions (rwxr-xr-x)
        if let Ok(metadata) = fs::metadata(target_binary) {
            let mut perms = metadata.permissions();
            perms.set_mode(perms.mode() | 0o755);
            let _ = fs::set_permissions(target_binary, perms);
        }

        xdg.ensure_dirs()?;
        let symlink_path = xdg.bin_dir().join(app_id);

        // If a file or symlink already exists, remove it first
        if symlink_path.exists() || symlink_path.is_symlink() {
            let _ = fs::remove_file(&symlink_path);
        }

        symlink(target_binary, &symlink_path)?;

        Ok(symlink_path)
    }

    /// Remove symlink from ~/.local/bin/
    pub fn remove_bin_symlink(xdg: &XdgPaths, app_id: &str) -> Result<()> {
        let symlink_path = xdg.bin_dir().join(app_id);
        if symlink_path.exists() || symlink_path.is_symlink() {
            let _ = fs::remove_file(&symlink_path);
        }
        Ok(())
    }
}

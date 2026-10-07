use std::fs;
use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::util::xdg::XdgPaths;

pub struct IconInstaller;

impl IconInstaller {
    /// Install extracted icon bytes or file into XDG icons hierarchy
    pub fn install_icon(
        xdg: &XdgPaths,
        app_id: &str,
        icon_bytes: Option<&[u8]>,
        source_icon_path: Option<&Path>,
        extension: Option<&str>,
    ) -> Result<Vec<PathBuf>> {
        let mut installed_paths = Vec::new();
        let ext = extension.unwrap_or("png");
        let icon_filename = format!("cart-{}.{}", app_id, ext);

        // Target directories: scalable for SVG, 256x256 for PNG
        let target_subdir = if ext == "svg" {
            xdg.icons_dir().join("scalable/apps")
        } else {
            xdg.icons_dir().join("256x256/apps")
        };

        fs::create_dir_all(&target_subdir)?;
        let target_icon_path = target_subdir.join(&icon_filename);

        if let Some(bytes) = icon_bytes {
            fs::write(&target_icon_path, bytes)?;
            installed_paths.push(target_icon_path.clone());
        } else if let Some(src) = source_icon_path
            && src.exists()
        {
            fs::copy(src, &target_icon_path)?;
            installed_paths.push(target_icon_path.clone());
        }

        // Also copy to ~/.local/share/pixmaps as fallback for some desktop environments
        let pixmaps_dir = xdg.data_home().join("pixmaps");
        if fs::create_dir_all(&pixmaps_dir).is_ok() {
            let pixmap_target = pixmaps_dir.join(&icon_filename);
            if target_icon_path.exists() {
                let _ = fs::copy(&target_icon_path, &pixmap_target);
                installed_paths.push(pixmap_target);
            }
        }

        Ok(installed_paths)
    }
}

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::util::xdg::XdgPaths;

pub struct IconInstaller;

impl IconInstaller {
    /// Install extracted icon bytes or file into XDG icons hierarchy across all desktop environments
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

        // 1. Determine primary target directory: scalable for SVG, 256x256 for PNG
        let primary_dir = if ext == "svg" {
            xdg.icons_dir().join("scalable/apps")
        } else {
            xdg.icons_dir().join("256x256/apps")
        };

        fs::create_dir_all(&primary_dir)?;
        let primary_icon_path = primary_dir.join(&icon_filename);

        if let Some(bytes) = icon_bytes {
            fs::write(&primary_icon_path, bytes)?;
            installed_paths.push(primary_icon_path.clone());
        } else if let Some(src) = source_icon_path
            && src.exists()
        {
            fs::copy(src, &primary_icon_path)?;
            installed_paths.push(primary_icon_path.clone());
        }

        if !primary_icon_path.exists() {
            return Ok(installed_paths);
        }

        // 2. Install to standard multi-resolution directories for different DE panels/docks (KDE, XFCE, GNOME)
        if ext == "png" {
            let resolution_dirs = [
                "512x512/apps",
                "128x128/apps",
                "64x64/apps",
                "48x48/apps",
                "32x32/apps",
            ];

            for res_dir in &resolution_dirs {
                let target_dir = xdg.icons_dir().join(res_dir);
                if fs::create_dir_all(&target_dir).is_ok() {
                    let dest = target_dir.join(&icon_filename);
                    if fs::copy(&primary_icon_path, &dest).is_ok() {
                        installed_paths.push(dest);
                    }
                }
            }
        }

        // 3. Copy to ~/.local/share/pixmaps/ as fallback for some desktop environments (e.g. XFCE, IceWM, Openbox)
        let pixmaps_dir = xdg.pixmaps_dir();
        if fs::create_dir_all(&pixmaps_dir).is_ok() {
            let pixmap_target = pixmaps_dir.join(&icon_filename);
            if fs::copy(&primary_icon_path, &pixmap_target).is_ok() {
                installed_paths.push(pixmap_target);
            }
        }

        // 4. Copy to ~/.icons/ as legacy X11 / GTK fallback
        let legacy_icons_dir = xdg.home_dir.join(".icons");
        if fs::create_dir_all(&legacy_icons_dir).is_ok() {
            let legacy_target = legacy_icons_dir.join(&icon_filename);
            if fs::copy(&primary_icon_path, &legacy_target).is_ok() {
                installed_paths.push(legacy_target);
            }
        }

        // 5. Copy to ~/.local/share/icons/ directly for flat icon lookups
        let share_icons_dir = xdg.data_home().join("icons");
        if fs::create_dir_all(&share_icons_dir).is_ok() {
            let direct_target = share_icons_dir.join(&icon_filename);
            if fs::copy(&primary_icon_path, &direct_target).is_ok() {
                installed_paths.push(direct_target);
            }
        }

        Ok(installed_paths)
    }
}

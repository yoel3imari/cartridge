pub mod desktop_file;
pub mod icon;
pub mod symlink;

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::extractor::squashfs::ExtractedMetadata;
use crate::integrator::desktop_file::DesktopEntryMutator;
use crate::integrator::icon::IconInstaller;
use crate::integrator::symlink::SymlinkManager;
use crate::util::xdg::XdgPaths;

#[derive(Debug, Clone)]
pub struct IntegrationResult {
    pub desktop_file: PathBuf,
    pub icon_paths: Vec<PathBuf>,
    pub symlink_path: PathBuf,
}

pub struct Integrator {
    xdg: XdgPaths,
}

impl Integrator {
    pub fn new() -> Self {
        Self {
            xdg: XdgPaths::new(),
        }
    }

    /// Perform full FreeDesktop integration for an installed AppImage
    pub fn integrate(
        &self,
        app_id: &str,
        app_name: &str,
        binary_path: &Path,
        extracted: &ExtractedMetadata,
        categories: &[String],
        description: Option<&str>,
    ) -> Result<IntegrationResult> {
        self.xdg.ensure_dirs()?;

        // 1. Install icons
        let icon_name = format!("cart-{}", app_id);
        let icon_paths = IconInstaller::install_icon(
            &self.xdg,
            app_id,
            extracted.icon_bytes.as_deref(),
            extracted.icon_path.as_deref(),
            extracted.icon_extension.as_deref(),
        )?;

        // Absolute path guarantees immediate display across all desktop environments
        let icon_entry = icon_paths
            .first()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or(icon_name);

        // 2. Generate and write desktop entry
        let desktop_content = DesktopEntryMutator::build_desktop_entry(
            extracted.desktop_content.as_deref(),
            app_id,
            app_name,
            binary_path,
            &icon_entry,
            categories,
            description,
        );

        let desktop_file = self
            .xdg
            .applications_dir()
            .join(format!("cart-{}.desktop", app_id));
        fs::write(&desktop_file, desktop_content)?;

        // 3. Create ~/.local/bin symlink
        let symlink_path = SymlinkManager::create_bin_symlink(&self.xdg, app_id, binary_path)?;

        // 4. Update desktop & icon databases if available
        self.refresh_system_databases();

        Ok(IntegrationResult {
            desktop_file,
            icon_paths,
            symlink_path,
        })
    }

    /// Remove desktop file, icons, and symlink
    pub fn unintegrate(
        &self,
        app_id: &str,
        desktop_file: Option<&Path>,
        icon_paths: &[PathBuf],
    ) -> Result<()> {
        // Remove symlink
        let _ = SymlinkManager::remove_bin_symlink(&self.xdg, app_id);

        // Remove desktop file
        if let Some(df) = desktop_file
            && df.exists()
        {
            let _ = fs::remove_file(df);
        }
        let fallback_df = self
            .xdg
            .applications_dir()
            .join(format!("cart-{}.desktop", app_id));
        if fallback_df.exists() {
            let _ = fs::remove_file(fallback_df);
        }
        let legacy_df = self
            .xdg
            .applications_dir()
            .join(format!("aim-{}.desktop", app_id));
        if legacy_df.exists() {
            let _ = fs::remove_file(legacy_df);
        }

        // Remove icons
        for icon in icon_paths {
            if icon.exists() {
                let _ = fs::remove_file(icon);
            }
        }

        // Additional cleanup for potential fallbacks
        for ext in &["png", "svg"] {
            let filename = format!("cart-{}.{}", app_id, ext);
            let _ = fs::remove_file(self.xdg.pixmaps_dir().join(&filename));
            let _ = fs::remove_file(self.xdg.home_dir.join(format!(".icons/{}", filename)));
            let _ = fs::remove_file(self.xdg.data_home().join(format!("icons/{}", filename)));
        }

        // Refresh system databases across GNOME, KDE, and other desktop environments
        self.refresh_system_databases();

        Ok(())
    }

    /// Trigger desktop environment database update commands
    pub fn refresh_system_databases(&self) {
        crate::util::desktop::refresh_desktop_databases(
            &self.xdg.applications_dir(),
            &self.xdg.icons_dir(),
            &self.xdg.mime_dir(),
        );
    }
}

impl Default for Integrator {
    fn default() -> Self {
        Self::new()
    }
}

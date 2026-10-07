pub mod state;

use std::fs;
use std::path::{Path, PathBuf};

use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, Table};

use crate::catalog::SearchService;
use crate::catalog::sanitize_id;
use crate::downloader::Downloader;
use crate::error::{AimError, Result};
use crate::extractor::AppImageExtractor;
use crate::integrator::Integrator;
use crate::manager::state::{InstalledApp, StateStore, now_iso};
use crate::util::xdg::XdgPaths;

pub struct AppManager {
    xdg: XdgPaths,
    state: StateStore,
    downloader: Downloader,
    integrator: Integrator,
    search_service: SearchService,
}

impl AppManager {
    pub fn new() -> Self {
        let xdg = XdgPaths::new();
        let _ = xdg.migrate_from_aim_if_needed();
        Self {
            xdg,
            state: StateStore::new(),
            downloader: Downloader::new(),
            integrator: Integrator::new(),
            search_service: SearchService::new(),
        }
    }

    /// Install an application from catalog name, GitHub repo, or URL
    pub async fn install(&self, target: &str, custom_id: Option<&str>) -> Result<InstalledApp> {
        self.xdg.ensure_dirs()?;

        // Case 0: Local file path on disk
        let expanded_path = if let Some(stripped) = target.strip_prefix("~/") {
            self.xdg.home_dir.join(stripped)
        } else {
            PathBuf::from(target)
        };

        if expanded_path.exists() || target.to_lowercase().ends_with(".appimage") {
            if expanded_path.exists() {
                return self.integrate_local_file(&expanded_path, custom_id);
            } else {
                return Err(AimError::NotFound(format!(
                    "Local file '{}' not found.",
                    target
                )));
            }
        }

        // Case 1: Direct HTTP URL
        if target.starts_with("http://") || target.starts_with("https://") {
            let file_name = target
                .split('?')
                .next()
                .unwrap_or(target)
                .split('/')
                .next_back()
                .unwrap_or("app.AppImage");

            let app_id = custom_id.map(|s| s.to_string()).unwrap_or_else(|| {
                sanitize_id(
                    file_name
                        .trim_end_matches(".AppImage")
                        .trim_end_matches(".appimage"),
                )
            });

            return self
                .download_and_install_file(target, &app_id, &app_id, "unknown", target, file_name)
                .await;
        }

        // Case 2: GitHub owner/repo
        if target.contains('/') && !target.starts_with('.') {
            let (release, asset) = self
                .search_service
                .github()
                .resolve_appimage(target)
                .await?;
            let app_id = custom_id
                .map(|s| s.to_string())
                .unwrap_or_else(|| sanitize_id(target.split('/').next_back().unwrap_or(target)));

            let app_name = target.split('/').next_back().unwrap_or(target);
            let version = release.tag_name;
            let source = format!("gh:{}", target);

            return self
                .download_and_install_file(
                    &asset.browser_download_url,
                    &app_id,
                    app_name,
                    &version,
                    &source,
                    &asset.name,
                )
                .await;
        }

        // Case 3: Search catalog for matching app
        let results = self.search_service.search(target, None, 5, false).await?;
        if results.is_empty() {
            return Err(AimError::NotFound(format!(
                "No AppImage found for query '{}'. Try 'cart search <query>' or specify a GitHub repo 'owner/repo'.",
                target
            )));
        }

        let best = &results[0];
        let app_id = custom_id
            .map(|s| s.to_string())
            .unwrap_or_else(|| best.id.clone());

        // Resolve download source
        if let Some(ref gh_repo) = best.github_repo
            && let Ok((release, asset)) =
                self.search_service.github().resolve_appimage(gh_repo).await
        {
            let source = format!("gh:{}", gh_repo);
            return self
                .download_and_install_file(
                    &asset.browser_download_url,
                    &app_id,
                    &best.name,
                    &release.tag_name,
                    &source,
                    &asset.name,
                )
                .await;
        }

        if let Some(ref dl_url) = best.download_url {
            let file_name = format!("{}.AppImage", app_id);
            return self
                .download_and_install_file(
                    dl_url,
                    &app_id,
                    &best.name,
                    "latest",
                    "AppImageHub",
                    &file_name,
                )
                .await;
        }

        Err(AimError::Install(format!(
            "Could not resolve a direct download asset for '{}'.",
            best.name
        )))
    }

    /// Internal routine: download, extract, integrate, save state
    async fn download_and_install_file(
        &self,
        url: &str,
        app_id: &str,
        app_name: &str,
        version: &str,
        source: &str,
        file_name: &str,
    ) -> Result<InstalledApp> {
        let app_dir = self.xdg.apps_dir().join(app_id);
        fs::create_dir_all(&app_dir)?;

        let final_binary = app_dir.join(file_name);
        println!("⬇️  Downloading {} ({})...", app_name, version);

        let (bytes_downloaded, sha256) = self
            .downloader
            .download_file(url, &final_binary, app_name)
            .await?;

        println!("📦 Extracting metadata & desktop assets...");
        let extracted = AppImageExtractor::extract_metadata(&final_binary)?;

        println!("🔗 Integrating desktop entry, icons, and command symlink...");
        let integration =
            self.integrator
                .integrate(app_id, app_name, &final_binary, &extracted, &[], None)?;

        let installed_app = InstalledApp {
            id: app_id.to_string(),
            name: app_name.to_string(),
            version: version.to_string(),
            source: source.to_string(),
            binary_path: final_binary,
            backup_binary_path: None,
            desktop_file: Some(integration.desktop_file),
            icon_paths: integration.icon_paths,
            symlink_path: Some(integration.symlink_path),
            installed_at: now_iso(),
            sha256: Some(sha256),
            file_size: bytes_downloaded,
        };

        self.state.register_app(installed_app.clone())?;
        println!(
            "✨ Successfully installed {} v{}! Run it with: {}",
            app_name, version, app_id
        );

        Ok(installed_app)
    }

    /// Integrate a local existing AppImage into the system
    pub fn integrate_local_file(
        &self,
        local_path: &Path,
        custom_id: Option<&str>,
    ) -> Result<InstalledApp> {
        if !local_path.exists() {
            return Err(AimError::NotFound(format!(
                "File does not exist: {}",
                local_path.display()
            )));
        }

        let file_name = local_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("app.AppImage");

        // Extract metadata first to discover real application name and icons
        let extracted = AppImageExtractor::extract_metadata(local_path)?;
        let discovered_name = extracted.desktop_content.as_deref().and_then(
            crate::integrator::desktop_file::DesktopEntryMutator::extract_name_from_desktop,
        );

        let fallback_name = file_name
            .trim_end_matches(".AppImage")
            .trim_end_matches(".appimage");
        let app_name = discovered_name.unwrap_or_else(|| fallback_name.to_string());

        let app_id = custom_id
            .map(|s| s.to_string())
            .unwrap_or_else(|| sanitize_id(&app_name));

        let app_dir = self.xdg.apps_dir().join(&app_id);
        fs::create_dir_all(&app_dir)?;

        let target_binary = app_dir.join(file_name);
        fs::copy(local_path, &target_binary)?;

        let metadata = fs::metadata(&target_binary)?;
        let file_size = metadata.len();

        let integration =
            self.integrator
                .integrate(&app_id, &app_name, &target_binary, &extracted, &[], None)?;

        let installed_app = InstalledApp {
            id: app_id.clone(),
            name: app_name.clone(),
            version: "local".to_string(),
            source: format!("local:{}", local_path.display()),
            binary_path: target_binary,
            backup_binary_path: None,
            desktop_file: Some(integration.desktop_file),
            icon_paths: integration.icon_paths,
            symlink_path: Some(integration.symlink_path),
            installed_at: now_iso(),
            sha256: None,
            file_size,
        };

        self.state.register_app(installed_app.clone())?;
        println!(
            "✨ Successfully integrated {}! Run it with: {}",
            app_name, app_id
        );

        Ok(installed_app)
    }

    /// List all installed applications
    pub fn list(&self) -> Result<()> {
        let apps = self.state.list_apps()?;
        if apps.is_empty() {
            println!(
                "No AppImages currently managed by cartridge. Install one using 'cart install <app>'!"
            );
            return Ok(());
        }

        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_header(vec![
                Cell::new("ID").fg(Color::Cyan),
                Cell::new("Name").fg(Color::Cyan),
                Cell::new("Version").fg(Color::Cyan),
                Cell::new("Size").fg(Color::Cyan),
                Cell::new("Source").fg(Color::Cyan),
                Cell::new("Command").fg(Color::Cyan),
            ]);

        for app in apps {
            let size_mb = format!("{:.1} MB", app.file_size as f64 / (1024.0 * 1024.0));
            let cmd = app
                .symlink_path
                .as_ref()
                .and_then(|p| p.file_name())
                .and_then(|f| f.to_str())
                .unwrap_or(&app.id);

            table.add_row(vec![
                Cell::new(&app.id).fg(Color::Green),
                Cell::new(&app.name),
                Cell::new(&app.version).fg(Color::Yellow),
                Cell::new(size_mb),
                Cell::new(&app.source),
                Cell::new(cmd).fg(Color::Cyan),
            ]);
        }

        println!("{table}");
        Ok(())
    }

    /// Show details for an installed or catalog application
    pub async fn info(&self, target: &str) -> Result<()> {
        // Check installed apps first
        if let Ok(Some(app)) = self.state.get_app(target) {
            println!("╭────────────────────────────────────────────────────────╮");
            println!("│ 📦 Application Information (Installed)                │");
            println!("╰────────────────────────────────────────────────────────╯");
            println!("  ID:          {}", app.id);
            println!("  Name:        {}", app.name);
            println!("  Version:     {}", app.version);
            println!(
                "  Size:        {:.1} MB",
                app.file_size as f64 / (1024.0 * 1024.0)
            );
            println!("  Source:      {}", app.source);
            println!("  Binary:      {}", app.binary_path.display());
            if let Some(ref symlink) = app.symlink_path {
                println!("  Symlink:     {}", symlink.display());
            }
            if let Some(ref desktop) = app.desktop_file {
                println!("  Desktop:     {}", desktop.display());
            }
            if let Some(ref backup) = app.backup_binary_path {
                println!("  Rollback:    {}", backup.display());
            }
            if let Some(ref sha) = app.sha256 {
                println!("  SHA256:      {}", sha);
            }
            println!("  Installed:   {}", app.installed_at);
            return Ok(());
        }

        // Search catalog
        let results = self.search_service.search(target, None, 1, false).await?;
        if let Some(best) = results.first() {
            println!("╭────────────────────────────────────────────────────────╮");
            println!("│ 🔍 Application Information (Catalog)                  │");
            println!("╰────────────────────────────────────────────────────────╯");
            println!("  Name:        {}", best.name);
            println!("  Description: {}", best.description);
            println!("  Categories:  {}", best.categories.join(", "));
            println!("  Source:      {}", best.source);
            if let Some(ref gh) = best.github_repo {
                println!("  GitHub:      https://github.com/{}", gh);
            }
            if let Some(ref dl) = best.download_url {
                println!("  Download:    {}", dl);
            }
            return Ok(());
        }

        Err(AimError::NotFound(format!(
            "Application '{}' not found",
            target
        )))
    }

    /// Remove an installed application
    pub fn remove(&self, app_id: &str, purge: bool) -> Result<()> {
        let app = self.state.get_app(app_id)?.ok_or_else(|| {
            AimError::NotFound(format!("Application '{}' is not installed", app_id))
        })?;

        println!("🗑️  Removing {}...", app.name);

        // Remove desktop integration
        self.integrator
            .unintegrate(&app.id, app.desktop_file.as_deref(), &app.icon_paths)?;

        // Remove app directory
        let app_dir = self.xdg.apps_dir().join(&app.id);
        if app_dir.exists() {
            let _ = fs::remove_dir_all(&app_dir);
        }

        // Unregister from state
        self.state.unregister_app(&app.id)?;

        if purge {
            let config_dir = self.xdg.config_home().join(&app.id);
            if config_dir.exists() {
                println!(
                    "🧹 Purging configuration directory: {}",
                    config_dir.display()
                );
                let _ = fs::remove_dir_all(config_dir);
            }
        }

        println!("✨ Successfully removed {}!", app.name);
        Ok(())
    }

    /// Update one or all installed applications
    pub async fn update(&self, app_id: Option<&str>) -> Result<()> {
        let apps_to_update = match app_id {
            Some(id) => {
                let app = self.state.get_app(id)?.ok_or_else(|| {
                    AimError::NotFound(format!("Application '{}' is not installed", id))
                })?;
                vec![app]
            }
            None => self.state.list_apps()?,
        };

        if apps_to_update.is_empty() {
            println!("No applications installed to update.");
            return Ok(());
        }

        for mut app in apps_to_update {
            if !app.source.starts_with("gh:") {
                println!(
                    "ℹ️  Skipping '{}' (source '{}' does not support auto-update yet)",
                    app.name, app.source
                );
                continue;
            }

            let gh_repo = app.source.trim_start_matches("gh:");
            println!(
                "🔍 Checking updates for {} (current: {})...",
                app.name, app.version
            );

            match self.search_service.github().resolve_appimage(gh_repo).await {
                Ok((release, asset)) => {
                    if release.tag_name == app.version {
                        println!("✅ {} is already up to date ({})", app.name, app.version);
                        continue;
                    }

                    println!(
                        "⬆️  Updating {} from {} -> {}...",
                        app.name, app.version, release.tag_name
                    );

                    let app_dir = self.xdg.apps_dir().join(&app.id);
                    let backup_file = app_dir.join(format!(
                        "{}.old",
                        app.binary_path
                            .file_name()
                            .and_then(|f| f.to_str())
                            .unwrap_or("app.AppImage")
                    ));
                    let new_file = app_dir.join(&asset.name);

                    // Download new version
                    let (size, sha256) = self
                        .downloader
                        .download_file(&asset.browser_download_url, &new_file, &app.name)
                        .await?;

                    // Move old binary to backup
                    if app.binary_path.exists() {
                        let _ = fs::rename(&app.binary_path, &backup_file);
                    }

                    // Re-integrate
                    let extracted = AppImageExtractor::extract_metadata(&new_file)?;
                    let integration = self.integrator.integrate(
                        &app.id,
                        &app.name,
                        &new_file,
                        &extracted,
                        &[],
                        None,
                    )?;

                    // Update state
                    app.version = release.tag_name;
                    app.binary_path = new_file;
                    app.backup_binary_path = Some(backup_file);
                    app.file_size = size;
                    app.sha256 = Some(sha256);
                    app.desktop_file = Some(integration.desktop_file);
                    app.icon_paths = integration.icon_paths;
                    app.symlink_path = Some(integration.symlink_path);

                    self.state.register_app(app.clone())?;
                    println!("✨ Updated {} to {}!", app.name, app.version);
                }
                Err(e) => {
                    println!("⚠️  Failed to check update for {}: {}", app.name, e);
                }
            }
        }

        Ok(())
    }

    /// Rollback an application to its previous version
    pub fn rollback(&self, app_id: &str) -> Result<()> {
        let mut app = self.state.get_app(app_id)?.ok_or_else(|| {
            AimError::NotFound(format!("Application '{}' is not installed", app_id))
        })?;

        let backup_path = app.backup_binary_path.as_ref().ok_or_else(|| {
            AimError::Other(format!(
                "No previous rollback version available for '{}'",
                app_id
            ))
        })?;

        if !backup_path.exists() {
            return Err(AimError::Other(format!(
                "Rollback file not found at {}",
                backup_path.display()
            )));
        }

        println!("⏪ Rolling back {}...", app.name);

        let restored_name = backup_path
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("app.AppImage")
            .trim_end_matches(".old");

        let restored_binary = app.binary_path.parent().unwrap().join(restored_name);
        fs::rename(backup_path, &restored_binary)?;

        // Re-integrate restored binary
        let extracted = AppImageExtractor::extract_metadata(&restored_binary)?;
        let integration = self.integrator.integrate(
            &app.id,
            &app.name,
            &restored_binary,
            &extracted,
            &[],
            None,
        )?;

        app.binary_path = restored_binary;
        app.backup_binary_path = None;
        app.version = format!("{}-rollback", app.version);
        app.desktop_file = Some(integration.desktop_file);
        app.icon_paths = integration.icon_paths;
        app.symlink_path = Some(integration.symlink_path);

        self.state.register_app(app.clone())?;
        println!("✨ Successfully rolled back {}!", app.name);

        Ok(())
    }

    /// Clean broken symlinks, orphaned desktop files, and stale backups
    pub fn clean(&self) -> Result<()> {
        let bin_dir = self.xdg.bin_dir();
        let apps_dir = self.xdg.apps_dir();
        let legacy_apps_dir = self.xdg.legacy_aim_data_dir().join("apps");
        let mut cleaned_items = 0;

        if bin_dir.exists() {
            for entry in fs::read_dir(&bin_dir)?.flatten() {
                let path = entry.path();
                if path.is_symlink()
                    && let Ok(target) = fs::read_link(&path)
                {
                    // Check if pointing to cartridge or legacy aim directory and destination is gone
                    if (target.starts_with(&apps_dir) || target.starts_with(&legacy_apps_dir))
                        && !target.exists()
                    {
                        println!("🧹 Removing broken symlink: {}", path.display());
                        let _ = fs::remove_file(&path);
                        cleaned_items += 1;
                    }
                }
            }
        }

        println!(
            "✨ Cleanup complete! Removed {} orphaned items.",
            cleaned_items
        );
        Ok(())
    }

    /// Search across installed apps and remote catalog
    pub async fn search(
        &self,
        query: &str,
        limit: usize,
        force_refresh: bool,
    ) -> Result<Vec<crate::catalog::models::SearchResult>> {
        let installed = self.state.list_apps().unwrap_or_default();
        self.search_service
            .search(query, Some(&installed), limit, force_refresh)
            .await
    }

    pub fn state(&self) -> &StateStore {
        &self.state
    }

    pub fn search_service(&self) -> &SearchService {
        &self.search_service
    }
}

impl Default for AppManager {
    fn default() -> Self {
        Self::new()
    }
}

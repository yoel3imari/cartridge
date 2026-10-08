use std::path::PathBuf;

pub struct XdgPaths {
    pub home_dir: PathBuf,
}

impl XdgPaths {
    pub fn new() -> Self {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/tmp"));
        Self { home_dir: home }
    }

    pub fn data_home(&self) -> PathBuf {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| self.home_dir.join(".local/share"))
    }

    pub fn cache_home(&self) -> PathBuf {
        std::env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| self.home_dir.join(".cache"))
    }

    pub fn config_home(&self) -> PathBuf {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| self.home_dir.join(".config"))
    }

    /// Root directory for cartridge application state: ~/.local/share/cartridge
    pub fn cartridge_data_dir(&self) -> PathBuf {
        self.data_home().join("cartridge")
    }

    /// Legacy root data directory: ~/.local/share/aim
    pub fn legacy_data_dir(&self) -> PathBuf {
        self.data_home().join("aim")
    }

    /// Root directory where cartridge stores installed apps: ~/.local/share/cartridge/apps/
    pub fn apps_dir(&self) -> PathBuf {
        self.cartridge_data_dir().join("apps")
    }

    /// State file tracking installed apps: ~/.local/share/cartridge/installed.json
    pub fn state_file(&self) -> PathBuf {
        self.cartridge_data_dir().join("installed.json")
    }

    /// User bin directory: ~/.local/bin
    pub fn bin_dir(&self) -> PathBuf {
        self.home_dir.join(".local/bin")
    }

    /// FreeDesktop applications directory: ~/.local/share/applications/
    pub fn applications_dir(&self) -> PathBuf {
        self.data_home().join("applications")
    }

    /// FreeDesktop hicolor icons base directory: ~/.local/share/icons/hicolor/
    pub fn icons_dir(&self) -> PathBuf {
        self.data_home().join("icons/hicolor")
    }

    /// Cache directory for cartridge catalog feeds: ~/.cache/cartridge/
    pub fn cartridge_cache_dir(&self) -> PathBuf {
        self.cache_home().join("cartridge")
    }

    /// Cache directory alias
    pub fn cache_dir(&self) -> PathBuf {
        self.cartridge_cache_dir()
    }

    /// FreeDesktop mime directory: ~/.local/share/mime/
    pub fn mime_dir(&self) -> PathBuf {
        self.data_home().join("mime")
    }

    /// FreeDesktop pixmaps directory: ~/.local/share/pixmaps/
    pub fn pixmaps_dir(&self) -> PathBuf {
        self.data_home().join("pixmaps")
    }

    /// Automatically migrate legacy data to cartridge if present
    pub fn migrate_legacy_data_if_needed(&self) -> std::io::Result<()> {
        let legacy_data = self.legacy_data_dir();
        let new_data = self.cartridge_data_dir();

        // 1. Migrate ~/.local/share/aim -> ~/.local/share/cartridge
        if legacy_data.exists() && !legacy_data.is_symlink() {
            if !new_data.exists() {
                std::fs::rename(&legacy_data, &new_data)?;
                #[cfg(unix)]
                let _ = std::os::unix::fs::symlink(&new_data, &legacy_data);
            } else if !self.state_file().exists() && legacy_data.join("installed.json").exists() {
                let legacy_apps = legacy_data.join("apps");
                let new_apps = self.apps_dir();
                if legacy_apps.exists() && !new_apps.exists() {
                    let _ = std::fs::rename(&legacy_apps, &new_apps);
                }
                let legacy_state = legacy_data.join("installed.json");
                if legacy_state.exists() && !self.state_file().exists() {
                    let _ = std::fs::rename(&legacy_state, self.state_file());
                }
                let _ = std::fs::remove_dir_all(&legacy_data);
                #[cfg(unix)]
                let _ = std::os::unix::fs::symlink(&new_data, &legacy_data);
            }
        }

        // 2. Migrate ~/.cache/aim -> ~/.cache/cartridge
        let legacy_cache = self.cache_home().join("aim");
        let new_cache = self.cartridge_cache_dir();
        if legacy_cache.exists() && !legacy_cache.is_symlink() {
            if !new_cache.exists() {
                std::fs::rename(&legacy_cache, &new_cache)?;
                #[cfg(unix)]
                let _ = std::os::unix::fs::symlink(&new_cache, &legacy_cache);
            } else {
                let _ = std::fs::remove_dir_all(&legacy_cache);
                #[cfg(unix)]
                let _ = std::os::unix::fs::symlink(&new_cache, &legacy_cache);
            }
        }

        // 3. Migrate installed.json paths and desktop files
        let state_path = self.state_file();
        if state_path.exists()
            && let Ok(content) = std::fs::read_to_string(&state_path)
            && (content.contains("/aim/") || content.contains("aim-"))
            && let Ok(mut json) = serde_json::from_str::<serde_json::Value>(&content)
        {
            if let Some(apps) = json.get_mut("apps").and_then(|a| a.as_object_mut()) {
                for (app_id, app_val) in apps.iter_mut() {
                    // Update binary_path
                    if let Some(bin) = app_val.get("binary_path").and_then(|v| v.as_str())
                        && bin.contains("/.local/share/aim/")
                    {
                        let new_bin = bin.replace("/.local/share/aim/", "/.local/share/cartridge/");
                        app_val["binary_path"] = serde_json::Value::String(new_bin);
                    }
                    // Update backup_binary_path
                    if let Some(bak) = app_val.get("backup_binary_path").and_then(|v| v.as_str())
                        && bak.contains("/.local/share/aim/")
                    {
                        let new_bak = bak.replace("/.local/share/aim/", "/.local/share/cartridge/");
                        app_val["backup_binary_path"] = serde_json::Value::String(new_bak);
                    }
                    // Update desktop_file
                    if let Some(df) = app_val.get("desktop_file").and_then(|v| v.as_str()) {
                        let old_df_path = PathBuf::from(df);
                        let new_df_path = self
                            .applications_dir()
                            .join(format!("cart-{}.desktop", app_id));
                        if old_df_path.exists()
                            && let Ok(df_content) = std::fs::read_to_string(&old_df_path)
                        {
                            let updated_content = df_content
                                .replace("/.local/share/aim/", "/.local/share/cartridge/")
                                .replace(&format!("aim-{}", app_id), &format!("cart-{}", app_id));
                            let _ = std::fs::write(&new_df_path, updated_content);
                            let _ = std::fs::remove_file(&old_df_path);
                        }
                        app_val["desktop_file"] =
                            serde_json::Value::String(new_df_path.to_string_lossy().to_string());
                    }
                    // Update icon_paths
                    if let Some(icons) =
                        app_val.get_mut("icon_paths").and_then(|v| v.as_array_mut())
                    {
                        for icon_val in icons.iter_mut() {
                            if let Some(icon_str) = icon_val.as_str() {
                                let old_icon_path = PathBuf::from(icon_str);
                                let old_name = old_icon_path
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("");
                                if old_name.starts_with(&format!("aim-{}", app_id)) {
                                    let new_name = old_name.replacen("aim-", "cart-", 1);
                                    let new_icon_path = old_icon_path.with_file_name(&new_name);
                                    if old_icon_path.exists() {
                                        let _ = std::fs::rename(&old_icon_path, &new_icon_path);
                                    }
                                    *icon_val = serde_json::Value::String(
                                        new_icon_path.to_string_lossy().to_string(),
                                    );
                                }
                            }
                        }
                    }
                    // Update symlink
                    if let Some(symlink) = app_val.get("symlink_path").and_then(|v| v.as_str()) {
                        let symlink_path = PathBuf::from(symlink);
                        if let Some(target_bin) =
                            app_val.get("binary_path").and_then(|v| v.as_str())
                        {
                            let target_path = PathBuf::from(target_bin);
                            if target_path.exists() {
                                let _ = std::fs::remove_file(&symlink_path);
                                #[cfg(unix)]
                                let _ = std::os::unix::fs::symlink(&target_path, &symlink_path);
                            }
                        }
                    }
                }
            }
            if let Ok(new_json) = serde_json::to_string_pretty(&json) {
                let _ = std::fs::write(&state_path, new_json);
            }
        }

        Ok(())
    }

    /// Ensure that standard cartridge directories exist
    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        let _ = self.migrate_legacy_data_if_needed();
        std::fs::create_dir_all(self.apps_dir())?;
        std::fs::create_dir_all(self.bin_dir())?;
        std::fs::create_dir_all(self.applications_dir())?;
        std::fs::create_dir_all(self.icons_dir())?;
        std::fs::create_dir_all(self.icons_dir().join("scalable/apps"))?;
        std::fs::create_dir_all(self.icons_dir().join("256x256/apps"))?;
        std::fs::create_dir_all(self.pixmaps_dir())?;
        std::fs::create_dir_all(self.cartridge_cache_dir())?;
        Ok(())
    }
}

impl Default for XdgPaths {
    fn default() -> Self {
        Self::new()
    }
}

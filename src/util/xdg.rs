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

    /// Root directory where aim stores installed apps: ~/.local/share/aim/apps/
    pub fn apps_dir(&self) -> PathBuf {
        self.data_home().join("aim/apps")
    }

    /// State file tracking installed apps: ~/.local/share/aim/installed.json
    pub fn state_file(&self) -> PathBuf {
        self.data_home().join("aim/installed.json")
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

    /// Cache directory for aim catalog feeds: ~/.cache/aim/
    pub fn aim_cache_dir(&self) -> PathBuf {
        self.cache_home().join("aim")
    }

    /// Ensure that standard aim directories exist
    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(self.apps_dir())?;
        std::fs::create_dir_all(self.bin_dir())?;
        std::fs::create_dir_all(self.applications_dir())?;
        std::fs::create_dir_all(self.icons_dir())?;
        std::fs::create_dir_all(self.aim_cache_dir())?;
        Ok(())
    }
}

impl Default for XdgPaths {
    fn default() -> Self {
        Self::new()
    }
}

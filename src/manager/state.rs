use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::util::xdg::XdgPaths;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledApp {
    pub id: String,
    pub name: String,
    pub version: String,
    pub source: String,
    pub binary_path: PathBuf,
    pub backup_binary_path: Option<PathBuf>,
    pub desktop_file: Option<PathBuf>,
    pub icon_paths: Vec<PathBuf>,
    pub symlink_path: Option<PathBuf>,
    pub installed_at: String,
    pub sha256: Option<String>,
    pub file_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppRegistry {
    pub apps: HashMap<String, InstalledApp>,
}

pub struct StateStore {
    xdg: XdgPaths,
}

impl StateStore {
    pub fn new() -> Self {
        Self {
            xdg: XdgPaths::new(),
        }
    }

    pub fn state_file(&self) -> PathBuf {
        self.xdg.state_file()
    }

    pub fn load(&self) -> Result<AppRegistry> {
        let path = self.state_file();
        if !path.exists() {
            return Ok(AppRegistry::default());
        }

        let content = fs::read_to_string(&path)?;
        let registry: AppRegistry = serde_json::from_str(&content).unwrap_or_default();
        Ok(registry)
    }

    pub fn save(&self, registry: &AppRegistry) -> Result<()> {
        let path = self.state_file();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(registry)?;
        fs::write(&path, content)?;
        Ok(())
    }

    pub fn register_app(&self, app: InstalledApp) -> Result<()> {
        let mut registry = self.load()?;
        registry.apps.insert(app.id.clone(), app);
        self.save(&registry)
    }

    pub fn unregister_app(&self, app_id: &str) -> Result<Option<InstalledApp>> {
        let mut registry = self.load()?;
        let removed = registry.apps.remove(app_id);
        self.save(&registry)?;
        Ok(removed)
    }

    pub fn get_app(&self, app_id: &str) -> Result<Option<InstalledApp>> {
        let registry = self.load()?;
        Ok(registry.apps.get(app_id).cloned())
    }

    pub fn list_apps(&self) -> Result<Vec<InstalledApp>> {
        let registry = self.load()?;
        let mut apps: Vec<InstalledApp> = registry.apps.into_values().collect();
        apps.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(apps)
    }
}

impl Default for StateStore {
    fn default() -> Self {
        Self::new()
    }
}

pub fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

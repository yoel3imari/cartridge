use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use crate::catalog::models::{AppImageHubFeed, AppImageHubFeedRaw, FeedItem};
use crate::error::{AimError, Result};
use crate::util::xdg::XdgPaths;

pub const FEED_URL: &str = "https://appimage.github.io/feed.json";
pub const CACHE_TTL: Duration = Duration::from_secs(86400); // 24 hours

pub struct AppImageHubCatalog {
    xdg: XdgPaths,
}

impl Default for AppImageHubCatalog {
    fn default() -> Self {
        Self::new()
    }
}

impl AppImageHubCatalog {
    pub fn new() -> Self {
        Self {
            xdg: XdgPaths::new(),
        }
    }

    pub fn cache_file_path(&self) -> PathBuf {
        self.xdg.aim_cache_dir().join("appimagehub_feed.json")
    }

    /// Load catalog from cache if valid, or download fresh copy
    pub async fn load_or_fetch(&self, force_refresh: bool) -> Result<AppImageHubFeed> {
        let cache_path = self.cache_file_path();

        if !force_refresh
            && cache_path.exists()
            && let Ok(metadata) = fs::metadata(&cache_path)
            && let Ok(modified) = metadata.modified()
            && let Ok(age) = SystemTime::now().duration_since(modified)
            && age < CACHE_TTL
            && let Ok(content) = fs::read_to_string(&cache_path)
            && let Ok(raw) = serde_json::from_str::<AppImageHubFeedRaw>(&content)
        {
            return Ok(AppImageHubFeed::from_raw(raw));
        }

        self.fetch_and_cache().await
    }

    /// Explicitly fetch fresh catalog and write to cache
    pub async fn fetch_and_cache(&self) -> Result<AppImageHubFeed> {
        self.xdg.ensure_dirs()?;
        let cache_path = self.cache_file_path();

        let client = reqwest::Client::builder()
            .user_agent("aim-appimage-manager/0.1")
            .timeout(Duration::from_secs(30))
            .build()?;

        let response =
            client.get(FEED_URL).send().await.map_err(|e| {
                AimError::Catalog(format!("Failed to download AppImageHub feed: {e}"))
            })?;

        if !response.status().is_success() {
            return Err(AimError::Catalog(format!(
                "Failed to fetch feed, HTTP status: {}",
                response.status()
            )));
        }

        let body = response
            .text()
            .await
            .map_err(|e| AimError::Catalog(format!("Failed to read feed response body: {e}")))?;

        // Cache the raw content to disk
        let _ = fs::write(&cache_path, &body);

        let raw: AppImageHubFeedRaw = serde_json::from_str(&body).map_err(|e| {
            AimError::Catalog(format!("Failed to parse AppImageHub feed JSON: {e}"))
        })?;

        Ok(AppImageHubFeed::from_raw(raw))
    }

    /// Find an item by exact or normalized name
    pub async fn find_item(&self, name: &str) -> Result<Option<FeedItem>> {
        let feed = self.load_or_fetch(false).await?;
        let norm = name.trim().to_lowercase();

        for item in feed.items {
            if item.get_name().to_lowercase() == norm {
                return Ok(Some(item));
            }
        }

        Ok(None)
    }
}

use std::time::Duration;

use crate::catalog::models::{GitHubRelease, GitHubReleaseAsset};
use crate::error::{AimError, Result};
use crate::util::arch::{asset_score, current_arch, matches_arch};

pub struct GitHubClient {
    client: reqwest::Client,
}

impl GitHubClient {
    pub fn new() -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::USER_AGENT,
            reqwest::header::HeaderValue::from_static("cartridge/0.1.0"),
        );
        headers.insert(
            reqwest::header::ACCEPT,
            reqwest::header::HeaderValue::from_static("application/vnd.github.v3+json"),
        );

        if let Ok(token) = std::env::var("GITHUB_TOKEN")
            && let Ok(val) =
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token.trim()))
        {
            headers.insert(reqwest::header::AUTHORIZATION, val);
        }

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        Self { client }
    }

    /// Fetch latest release for owner/repo
    pub async fn get_latest_release(&self, owner_repo: &str) -> Result<GitHubRelease> {
        let clean = owner_repo
            .trim_start_matches("https://github.com/")
            .trim_matches('/');
        let parts: Vec<&str> = clean.split('/').collect();
        if parts.len() < 2 {
            return Err(AimError::Catalog(format!(
                "Invalid GitHub repository identifier '{owner_repo}', expected 'owner/repo'"
            )));
        }

        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            parts[0], parts[1]
        );
        let resp = self.client.get(&url).send().await?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            // Try fetching releases list and pick the first non-draft
            let releases_url = format!(
                "https://api.github.com/repos/{}/{}/releases",
                parts[0], parts[1]
            );
            let list_resp = self.client.get(&releases_url).send().await?;
            if !list_resp.status().is_success() {
                return Err(AimError::NotFound(format!(
                    "No releases found for GitHub repo '{owner_repo}'"
                )));
            }
            let list: Vec<GitHubRelease> = list_resp.json().await?;
            return list.into_iter().next().ok_or_else(|| {
                AimError::NotFound(format!("No releases found for GitHub repo '{owner_repo}'"))
            });
        }

        if !resp.status().is_success() {
            return Err(AimError::Catalog(format!(
                "GitHub API returned HTTP {}: {}",
                resp.status(),
                resp.text().await.unwrap_or_default()
            )));
        }

        let release: GitHubRelease = resp.json().await?;
        Ok(release)
    }

    /// Find the best AppImage asset for the target arch from a release
    pub fn select_best_appimage<'a>(
        &self,
        assets: &'a [GitHubReleaseAsset],
        target_arch: &str,
    ) -> Option<&'a GitHubReleaseAsset> {
        let mut candidates: Vec<(&'a GitHubReleaseAsset, u32)> = assets
            .iter()
            .filter(|a| matches_arch(&a.name, target_arch))
            .map(|a| (a, asset_score(&a.name, target_arch)))
            .collect();

        candidates.sort_by_key(|b| std::cmp::Reverse(b.1));
        candidates.into_iter().next().map(|(asset, _)| asset)
    }

    /// Resolve an AppImage asset directly from owner/repo
    pub async fn resolve_appimage(
        &self,
        owner_repo: &str,
    ) -> Result<(GitHubRelease, GitHubReleaseAsset)> {
        let release = self.get_latest_release(owner_repo).await?;
        let arch = current_arch();
        let asset = self
            .select_best_appimage(&release.assets, arch)
            .cloned()
            .ok_or_else(|| {
                AimError::NotFound(format!(
                    "No AppImage asset found for architecture '{}' in {} release {}",
                    arch, owner_repo, release.tag_name
                ))
            })?;

        Ok((release, asset))
    }
}

impl Default for GitHubClient {
    fn default() -> Self {
        Self::new()
    }
}

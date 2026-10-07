pub mod appimagehub;
pub mod github;
pub mod models;

use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

use crate::catalog::appimagehub::AppImageHubCatalog;
use crate::catalog::github::GitHubClient;
use crate::catalog::models::SearchResult;
use crate::error::Result;
use crate::manager::state::InstalledApp;

pub struct SearchService {
    appimagehub: AppImageHubCatalog,
    github: GitHubClient,
    matcher: SkimMatcherV2,
}

impl SearchService {
    pub fn new() -> Self {
        Self {
            appimagehub: AppImageHubCatalog::new(),
            github: GitHubClient::new(),
            matcher: SkimMatcherV2::default(),
        }
    }

    /// Search across installed apps and AppImageHub catalog
    pub async fn search(
        &self,
        query: &str,
        installed_apps: Option<&[InstalledApp]>,
        limit: usize,
        force_refresh: bool,
    ) -> Result<Vec<SearchResult>> {
        let q = query.trim();
        let mut results = Vec::new();

        // 1. Search locally installed apps first
        if let Some(apps) = installed_apps {
            for app in apps {
                let mut score: i64 = 0;
                if let Some(s) = self.matcher.fuzzy_match(&app.name, q) {
                    score += s * 4;
                } else if app.name.to_lowercase().contains(&q.to_lowercase()) {
                    score += 60;
                }

                if let Some(s) = self.matcher.fuzzy_match(&app.id, q) {
                    score += s * 4;
                } else if app.id.to_lowercase().contains(&q.to_lowercase()) {
                    score += 60;
                }

                if score > 0 {
                    results.push(SearchResult {
                        id: app.id.clone(),
                        name: app.name.clone(),
                        description: format!("Installed application (v{})", app.version),
                        categories: vec!["Installed".to_string()],
                        source: "Installed".to_string(),
                        download_url: None,
                        github_repo: None,
                        score: score + 1000, // Prioritize installed apps
                    });
                }
            }
        }

        // 2. Search online catalog
        let feed = self.appimagehub.load_or_fetch(force_refresh).await?;

        for item in &feed.items {
            let item_name = item.get_name();
            let mut score: i64 = 0;

            // Score on name
            if let Some(s) = self.matcher.fuzzy_match(item_name, q) {
                score += s * 3;
            } else if item_name.to_lowercase().contains(&q.to_lowercase()) {
                score += 50;
            }

            // Score on description
            if let Some(ref desc) = item.description {
                if let Some(s) = self.matcher.fuzzy_match(desc, q) {
                    score += s;
                } else if desc.to_lowercase().contains(&q.to_lowercase()) {
                    score += 20;
                }
            }

            let categories = item.get_categories();
            // Score on categories
            for cat in &categories {
                if cat.to_lowercase() == q.to_lowercase() {
                    score += 30;
                }
            }

            if score > 0 {
                let links = item.get_links();
                let github_repo = links
                    .iter()
                    .find(|l| l.link_type.as_deref() == Some("GitHub"))
                    .and_then(|l| l.url.clone());

                let download_url = links
                    .iter()
                    .find(|l| l.link_type.as_deref() == Some("Download"))
                    .and_then(|l| l.url.clone());

                results.push(SearchResult {
                    id: sanitize_id(item_name),
                    name: item_name.to_string(),
                    description: item.description.clone().unwrap_or_default(),
                    categories,
                    source: "AppImageHub".to_string(),
                    download_url,
                    github_repo,
                    score,
                });
            }
        }

        results.sort_by_key(|b| std::cmp::Reverse(b.score));
        results.truncate(limit);

        // If query looks like an owner/repo and not found in catalog, check GitHub directly
        if results.is_empty()
            && q.contains('/')
            && !q.starts_with("http")
            && let Ok(release) = self.github.get_latest_release(q).await
        {
            results.push(SearchResult {
                id: sanitize_id(q.split('/').next_back().unwrap_or(q)),
                name: q.to_string(),
                description: release
                    .name
                    .unwrap_or_else(|| format!("Release {}", release.tag_name)),
                categories: vec!["GitHub".to_string()],
                source: "GitHub".to_string(),
                download_url: None,
                github_repo: Some(q.to_string()),
                score: 100,
            });
        }

        Ok(results)
    }

    pub fn appimagehub(&self) -> &AppImageHubCatalog {
        &self.appimagehub
    }

    pub fn github(&self) -> &GitHubClient {
        &self.github
    }
}

impl Default for SearchService {
    fn default() -> Self {
        Self::new()
    }
}

pub fn sanitize_id(name: &str) -> String {
    name.trim()
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

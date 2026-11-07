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

/// Shortest query (in characters) for which description matches are considered.
const MIN_DESCRIPTION_QUERY_LEN: usize = 3;

/// A fuzzy match may span at most this many characters per query character
/// before it is considered scattered noise (e.g. "qbit" matching
/// "PDF-Equilibrist" spans 11 characters for a 4 character query).
const FUZZY_SPAN_FACTOR: usize = 2;

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
            matcher: SkimMatcherV2::default().ignore_case(),
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
        if q.is_empty() {
            return Ok(Vec::new());
        }
        let q_l = q.to_lowercase();
        let mut results = Vec::new();

        // 1. Search locally installed apps first
        if let Some(apps) = installed_apps {
            for app in apps {
                let score = self.score_text(&app.name, &q_l).unwrap_or(0)
                    + self.score_text(&app.id, &q_l).unwrap_or(0);

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

            // Names and ids get tiered scoring with a density-checked fuzzy
            // fallback; descriptions only count on literal substring hits.
            let mut score = self.score_text(item_name, &q_l).unwrap_or(0);

            if let Some(ref desc) = item.description {
                score += score_description(desc, &q_l).unwrap_or(0);
            }

            let categories = item.get_categories();
            // Score on categories
            for cat in &categories {
                if cat.to_lowercase() == q_l {
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

    /// Score a name or id against the (lowercased) query.
    ///
    /// Tiers are ordered by match strength so a real substring hit always
    /// outranks a fuzzy one: exact (1000) > word-boundary substring (600) >
    /// substring (300) > dense fuzzy subsequence (at most 200).
    fn score_text(&self, text: &str, q_l: &str) -> Option<i64> {
        let text_l = text.to_lowercase();

        if text_l == q_l {
            return Some(1000);
        }

        if let Some(pos) = text_l.find(q_l) {
            return Some(if is_word_boundary(&text_l, pos) {
                600
            } else {
                300
            });
        }

        self.dense_fuzzy_score(text, q_l)
    }

    /// Typo-tolerant fallback for names and ids. A fuzzy subsequence only
    /// counts when its matched characters stay close together, so scattered
    /// hits like "qbit" matching "QUaModbusClient" are rejected as noise.
    fn dense_fuzzy_score(&self, text: &str, q_l: &str) -> Option<i64> {
        let query_len = q_l.chars().count();
        if query_len < 3 {
            return None;
        }

        let (score, indices) = self.matcher.fuzzy_indices(text, q_l)?;
        let first = *indices.first()?;
        let last = *indices.last()?;
        let span = last - first + 1;

        if span > query_len * FUZZY_SPAN_FACTOR {
            return None;
        }

        Some(score.min(200))
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

/// Score a description against the (lowercased) query. Only literal
/// substring hits count: fuzzy subsequence matching over free-form text
/// matches nearly everything and buries the real results.
fn score_description(desc: &str, q_l: &str) -> Option<i64> {
    if q_l.chars().count() < MIN_DESCRIPTION_QUERY_LEN {
        return None;
    }

    let desc_l = desc.to_lowercase();
    let pos = desc_l.find(q_l)?;

    Some(if is_word_boundary(&desc_l, pos) {
        150
    } else {
        80
    })
}

/// True when `pos` starts a word: the start of the text or preceded by a
/// non-alphanumeric character.
fn is_word_boundary(text: &str, pos: usize) -> bool {
    text[..pos]
        .chars()
        .next_back()
        .is_none_or(|c| !c.is_alphanumeric())
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

pub fn clean_app_name(file_name: &str) -> String {
    let base = file_name
        .trim_end_matches(".AppImage")
        .trim_end_matches(".appimage");

    let without_arch = base
        .replace("-linux-x86_64", "")
        .replace("-linux-amd64", "")
        .replace("-linux-aarch64", "")
        .replace("-linux-arm64", "")
        .replace("-x86_64", "")
        .replace("-amd64", "")
        .replace("-aarch64", "")
        .replace("-arm64", "")
        .replace("-i686", "")
        .replace(".x86_64", "")
        .replace(".amd64", "");

    if let Ok(re) = regex::Regex::new(r"[-_]v?\d+(\.\d+)*.*$") {
        let cleaned = re.replace(&without_arch, "").to_string();
        if !cleaned.is_empty() {
            return cleaned;
        }
    }

    without_arch
}

#[cfg(test)]
mod tests {
    use super::*;

    fn service() -> SearchService {
        SearchService::new()
    }

    #[test]
    fn name_scoring_tiers() {
        let s = service();
        let q = "qbit";

        assert_eq!(s.score_text("qbit", q), Some(1000), "exact match");
        assert_eq!(s.score_text("qBittorrent", q), Some(600), "word start");
        assert_eq!(s.score_text("myqbitapp", q), Some(300), "mid-word");
        assert_eq!(s.score_text("QUaModbusClient", q), None, "scattered");
        assert_eq!(s.score_text("PDF-Equilibrist", q), None, "spread out");
        assert_eq!(s.score_text("qTox", q), None, "no real match");
    }

    #[test]
    fn name_tiers_are_ordered() {
        let s = service();
        let q = "qbit";

        let exact = s.score_text("qbit", q).unwrap();
        let boundary = s.score_text("qbit-client", q).unwrap();
        let substring = s.score_text("myqbitapp", q).unwrap();
        let fuzzy = s.dense_fuzzy_score("qubittorrent", q).unwrap();

        assert!(
            exact > boundary && boundary > substring && substring > fuzzy,
            "exact ({exact}) > boundary ({boundary}) > substring ({substring}) > fuzzy ({fuzzy})"
        );
    }

    #[test]
    fn fuzzy_tolerates_typos_but_rejects_scattered_matches() {
        let s = service();
        let q = "qbit";

        // A single inserted letter keeps the match dense enough to count.
        assert!(s.dense_fuzzy_score("qubittorrent", q).is_some());
        // "qbit" spread across a long unrelated name must not match.
        assert!(s.dense_fuzzy_score("QUaModbusClient", q).is_none());
        // Short queries never fall back to fuzzy matching.
        assert!(s.dense_fuzzy_score("qTox", "q").is_none());
    }

    #[test]
    fn descriptions_only_count_literal_hits() {
        let q = "qbit";

        assert!(
            score_description(
                "Qt 5 based Tox instant messenger for secure communication",
                q
            )
            .is_none(),
            "fuzzy subsequences in descriptions must not match"
        );
        assert!(score_description("The qBittorrent project aims to provide", q).is_some());
        assert!(
            score_description("a tool for power users", "qt").is_none(),
            "queries below the minimum length ignore descriptions"
        );
    }
}

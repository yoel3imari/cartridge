use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedAuthor {
    pub name: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedLink {
    pub link_type: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FeedItem {
    pub name: Option<String>,
    pub description: Option<String>,
    pub categories: Vec<String>,
    pub authors: Vec<FeedAuthor>,
    pub license: Option<String>,
    pub links: Vec<FeedLink>,
    pub icons: Vec<String>,
}

impl FeedItem {
    pub fn from_value(val: &Value) -> Option<Self> {
        let obj = val.as_object()?;

        let name = obj.get("name").and_then(val_to_string);
        let description = obj.get("description").and_then(val_to_string);
        let license = obj.get("license").and_then(val_to_string);

        let mut categories = Vec::new();
        if let Some(cats) = obj.get("categories").and_then(|v| v.as_array()) {
            for c in cats {
                if let Some(s) = val_to_string(c) {
                    categories.push(s);
                }
            }
        }

        let mut authors = Vec::new();
        if let Some(auths) = obj.get("authors").and_then(|v| v.as_array()) {
            for a in auths {
                if let Some(a_obj) = a.as_object() {
                    let a_name = a_obj.get("name").and_then(val_to_string);
                    let a_url = a_obj.get("url").and_then(val_to_string);
                    authors.push(FeedAuthor {
                        name: a_name,
                        url: a_url,
                    });
                }
            }
        }

        let mut links = Vec::new();
        if let Some(ls) = obj.get("links").and_then(|v| v.as_array()) {
            for l in ls {
                if let Some(l_obj) = l.as_object() {
                    let l_type = l_obj.get("type").and_then(val_to_string);
                    let l_url = l_obj.get("url").and_then(val_to_string);
                    links.push(FeedLink {
                        link_type: l_type,
                        url: l_url,
                    });
                }
            }
        }

        let mut icons = Vec::new();
        if let Some(ics) = obj.get("icons").and_then(|v| v.as_array()) {
            for ic in ics {
                if let Some(s) = val_to_string(ic) {
                    icons.push(s);
                }
            }
        }

        Some(FeedItem {
            name,
            description,
            categories,
            authors,
            license,
            links,
            icons,
        })
    }

    pub fn get_name(&self) -> &str {
        self.name.as_deref().unwrap_or("Unknown")
    }

    pub fn get_categories(&self) -> Vec<String> {
        self.categories.clone()
    }

    pub fn get_links(&self) -> Vec<FeedLink> {
        self.links.clone()
    }
}

fn val_to_string(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppImageHubFeedRaw {
    #[serde(default)]
    pub version: Option<u32>,
    #[serde(default)]
    pub items: Vec<Value>,
}

#[derive(Debug, Clone)]
pub struct AppImageHubFeed {
    pub version: Option<u32>,
    pub items: Vec<FeedItem>,
}

impl AppImageHubFeed {
    pub fn from_raw(raw: AppImageHubFeedRaw) -> Self {
        let items = raw
            .items
            .into_iter()
            .filter_map(|v| FeedItem::from_value(&v))
            .collect();

        Self {
            version: raw.version,
            items,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub name: String,
    pub description: String,
    pub categories: Vec<String>,
    pub source: String,
    pub download_url: Option<String>,
    pub github_repo: Option<String>,
    pub score: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: Option<String>,
    pub draft: bool,
    pub prerelease: bool,
    #[serde(default)]
    pub assets: Vec<GitHubReleaseAsset>,
    pub body: Option<String>,
}

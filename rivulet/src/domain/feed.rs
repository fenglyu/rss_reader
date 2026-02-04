use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feed {
    pub id: i64,
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub last_fetched_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl Feed {
    pub fn new(url: String) -> Self {
        Self {
            id: 0,
            url,
            title: None,
            description: None,
            etag: None,
            last_modified: None,
            last_fetched_at: None,
            created_at: Utc::now(),
        }
    }

    pub fn display_title(&self) -> &str {
        self.title.as_deref().unwrap_or(&self.url)
    }
}

#[derive(Debug, Clone, Default)]
pub struct FeedUpdate {
    pub title: Option<String>,
    pub description: Option<String>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub last_fetched_at: Option<DateTime<Utc>>,
}

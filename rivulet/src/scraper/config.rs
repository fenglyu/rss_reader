use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for the web scraper
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ScraperConfig {
    /// Enable automatic background scraping (default: true)
    pub enabled: bool,

    /// Whether to run the browser in headless mode (default: true)
    pub headless: bool,

    /// Minimum content length to consider item as having content (default: 200)
    pub min_content_length: usize,

    /// Page load timeout in seconds (default: 30)
    pub timeout_secs: u64,

    /// Wait time after page load for dynamic content in milliseconds (default: 1000)
    pub wait_after_load_ms: u64,

    /// CSS selectors to try for article content extraction, in priority order
    pub content_selectors: Vec<String>,

    /// CSS selectors for elements to remove (ads, navigation, etc.)
    pub remove_selectors: Vec<String>,

    /// Maximum concurrent browser pages (default: 5)
    pub max_concurrency: usize,

    /// Whether to block images for faster loading (default: true)
    pub block_images: bool,

    /// Whether to block stylesheets for faster loading (default: true)
    pub block_stylesheets: bool,

    /// User agent string to use
    pub user_agent: Option<String>,
}

impl Default for ScraperConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            headless: true,
            min_content_length: 200,
            timeout_secs: 30,
            wait_after_load_ms: 1000,
            content_selectors: vec![
                // Common article content selectors in priority order
                "article".to_string(),
                "[role=\"main\"]".to_string(),
                "main".to_string(),
                ".post-content".to_string(),
                ".article-content".to_string(),
                ".entry-content".to_string(),
                ".content".to_string(),
                "#content".to_string(),
                ".post".to_string(),
                ".article".to_string(),
                ".blog-post".to_string(),
            ],
            remove_selectors: vec![
                // Common elements to remove
                "nav".to_string(),
                "header".to_string(),
                "footer".to_string(),
                "aside".to_string(),
                ".sidebar".to_string(),
                ".advertisement".to_string(),
                ".ad".to_string(),
                ".ads".to_string(),
                ".social-share".to_string(),
                ".comments".to_string(),
                ".related-posts".to_string(),
                "script".to_string(),
                "style".to_string(),
                "noscript".to_string(),
            ],
            max_concurrency: 5,
            block_images: true,
            block_stylesheets: true,
            user_agent: Some(
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 \
                 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
                    .to_string(),
            ),
        }
    }
}

impl ScraperConfig {
    /// Get the page load timeout as a Duration
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_secs)
    }

    /// Get the wait time after load as a Duration
    pub fn wait_after_load(&self) -> Duration {
        Duration::from_millis(self.wait_after_load_ms)
    }

    /// Create a config optimized for speed (less accurate)
    pub fn fast() -> Self {
        Self {
            timeout_secs: 15,
            wait_after_load_ms: 500,
            max_concurrency: 10,
            ..Default::default()
        }
    }

    /// Create a config optimized for accuracy (slower)
    pub fn thorough() -> Self {
        Self {
            timeout_secs: 60,
            wait_after_load_ms: 2000,
            max_concurrency: 3,
            block_images: false,
            block_stylesheets: false,
            ..Default::default()
        }
    }
}

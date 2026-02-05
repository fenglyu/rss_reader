//! Web scraping module for fetching full article content.
//!
//! This module provides browser-based content extraction for RSS feeds
//! that only include summaries or metadata without full article content.
//!
//! # Architecture
//!
//! ```text
//! Item (with link) → Scraper → Full content → Store update
//! ```
//!
//! # Usage
//!
//! ```rust,ignore
//! use rivulet::scraper::{ChromeScraper, Scraper, ScraperConfig};
//!
//! let config = ScraperConfig::default();
//! let scraper = ChromeScraper::new(config).await?;
//!
//! // Scrape a single URL
//! let content = scraper.scrape("https://example.com/article").await?;
//!
//! // Scrape multiple items in parallel
//! let results = scraper.scrape_items(&items, 5).await;
//! ```
//!
//! # Background Scraping
//!
//! ```rust,ignore
//! use rivulet::scraper::{spawn_background_scraper, ScraperConfig};
//!
//! let handle = spawn_background_scraper(config, store);
//! handle.queue_items(items).await;
//! ```

mod background;
mod chrome;
mod config;
mod extractor;

pub use background::{spawn_background_scraper, BackgroundScraperHandle};
pub use chrome::ChromeScraper;
pub use config::ScraperConfig;
pub use extractor::ContentExtractor;

use crate::app::Result;
use crate::domain::Item;
use async_trait::async_trait;

/// Result of a scraping operation
#[derive(Debug, Clone)]
pub struct ScrapeResult {
    /// The extracted article content (HTML or plain text)
    pub content: String,
    /// Whether the content is HTML (true) or plain text (false)
    pub is_html: bool,
}

/// Trait for web scraping implementations
#[async_trait]
pub trait Scraper: Send + Sync {
    /// Scrape content from a URL
    async fn scrape(&self, url: &str) -> Result<ScrapeResult>;

    /// Scrape content for multiple items concurrently
    ///
    /// Returns a vector of (item_id, Result<ScrapeResult>) pairs
    async fn scrape_items(
        &self,
        items: &[Item],
        concurrency: usize,
    ) -> Vec<(String, Result<ScrapeResult>)>;

    /// Check if an item needs content scraping
    ///
    /// Returns true if the item has a link but no meaningful content
    fn needs_scraping(item: &Item) -> bool {
        item.link.is_some() && item.content.as_ref().map(|c| c.len() < 200).unwrap_or(true)
    }
}

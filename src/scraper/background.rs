use std::sync::Arc;

use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::domain::Item;
use crate::scraper::{ChromeScraper, Scraper, ScraperConfig};
use crate::store::Store;

/// Message type for the background scraper
#[derive(Debug)]
pub enum ScrapeMessage {
    /// Scrape a batch of items
    ScrapeItems(Vec<Item>),
    /// Shutdown the scraper
    Shutdown,
}

/// Handle to send messages to the background scraper
#[derive(Clone)]
pub struct BackgroundScraperHandle {
    tx: mpsc::Sender<ScrapeMessage>,
}

impl BackgroundScraperHandle {
    /// Queue items for background scraping
    pub async fn queue_items(&self, items: Vec<Item>) {
        if items.is_empty() {
            return;
        }
        if let Err(e) = self.tx.send(ScrapeMessage::ScrapeItems(items)).await {
            warn!("Failed to queue items for scraping: {}", e);
        }
    }

    /// Shutdown the background scraper
    pub async fn shutdown(&self) {
        let _ = self.tx.send(ScrapeMessage::Shutdown).await;
    }
}

/// Background scraper service that processes items asynchronously
pub struct BackgroundScraper<S: Store + Send + Sync + 'static> {
    config: ScraperConfig,
    store: Arc<S>,
    rx: mpsc::Receiver<ScrapeMessage>,
}

impl<S: Store + Send + Sync + 'static> BackgroundScraper<S> {
    /// Create a new background scraper and return a handle to communicate with it
    pub fn new(config: ScraperConfig, store: Arc<S>) -> (Self, BackgroundScraperHandle) {
        let (tx, rx) = mpsc::channel(100);
        let handle = BackgroundScraperHandle { tx };
        let scraper = Self { config, store, rx };
        (scraper, handle)
    }

    /// Run the background scraper loop
    pub async fn run(mut self) {
        info!("Background scraper started");

        // Lazy initialization of browser - only when needed
        let mut scraper: Option<ChromeScraper> = None;

        while let Some(msg) = self.rx.recv().await {
            match msg {
                ScrapeMessage::ScrapeItems(items) => {
                    // Filter items that need scraping
                    let items_to_scrape: Vec<_> = items
                        .into_iter()
                        .filter(|item| self.needs_scraping(item))
                        .collect();

                    if items_to_scrape.is_empty() {
                        continue;
                    }

                    info!("Scraping {} items in background", items_to_scrape.len());

                    // Initialize scraper lazily
                    if scraper.is_none() {
                        match ChromeScraper::new(self.config.clone()).await {
                            Ok(s) => scraper = Some(s),
                            Err(e) => {
                                error!("Failed to initialize scraper: {}", e);
                                continue;
                            }
                        }
                    }

                    if let Some(ref s) = scraper {
                        let results = s
                            .scrape_items(&items_to_scrape, self.config.max_concurrency)
                            .await;

                        for (item_id, result) in results {
                            match result {
                                Ok(scrape_result) => {
                                    if let Err(e) = self
                                        .store
                                        .update_item_content(&item_id, &scrape_result.content)
                                    {
                                        error!("Failed to update item content: {}", e);
                                    } else {
                                        info!(
                                            "Scraped content for item {} ({} chars)",
                                            &item_id[..8],
                                            scrape_result.content.len()
                                        );
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to scrape item {}: {}", &item_id[..8], e);
                                }
                            }
                        }
                    }
                }
                ScrapeMessage::Shutdown => {
                    info!("Background scraper shutting down");
                    break;
                }
            }
        }
    }

    /// Check if an item needs scraping based on config
    fn needs_scraping(&self, item: &Item) -> bool {
        // Must have a link
        if item.link.is_none() {
            return false;
        }

        // Check if content is missing or too short
        match &item.content {
            None => true,
            Some(content) => content.len() < self.config.min_content_length,
        }
    }
}

/// Spawn the background scraper as a tokio task
pub fn spawn_background_scraper<S: Store + Send + Sync + 'static>(
    config: ScraperConfig,
    store: Arc<S>,
) -> BackgroundScraperHandle {
    let (scraper, handle) = BackgroundScraper::new(config, store);

    tokio::spawn(async move {
        scraper.run().await;
    });

    handle
}

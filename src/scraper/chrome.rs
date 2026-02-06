use std::sync::Arc;

use async_trait::async_trait;
use chromiumoxide::browser::{Browser, BrowserConfig};
use futures::StreamExt;
use tokio::sync::Semaphore;

use crate::app::{Result, RivuletError};
use crate::domain::Item;
use crate::scraper::config::ScraperConfig;
use crate::scraper::extractor::ContentExtractor;
use crate::scraper::{ScrapeResult, Scraper};

/// Chrome-based web scraper using chromiumoxide
pub struct ChromeScraper {
    browser: Arc<Browser>,
    config: ScraperConfig,
    extractor: ContentExtractor,
    semaphore: Arc<Semaphore>,
}

impl ChromeScraper {
    /// Create a new Chrome scraper with the given configuration
    pub async fn new(config: ScraperConfig) -> Result<Self> {
        let mut builder = BrowserConfig::builder()
            .arg("--no-sandbox")
            .arg("--disable-gpu")
            .arg("--disable-dev-shm-usage")
            .arg("--disable-software-rasterizer");

        if !config.headless {
            builder = builder.with_head();
        }

        let browser_config = builder
            .build()
            .map_err(|e| RivuletError::Scraper(format!("Failed to build browser config: {}", e)))?;

        let (browser, mut handler) = Browser::launch(browser_config)
            .await
            .map_err(|e| RivuletError::Scraper(format!(
                "Failed to launch browser: {}. Is Chrome or Chromium installed and in PATH?",
                e
            )))?;

        // Spawn the browser handler
        tokio::spawn(async move {
            while let Some(_event) = handler.next().await {
                // Handle browser events
            }
        });

        let semaphore = Arc::new(Semaphore::new(config.max_concurrency));
        let extractor = ContentExtractor::new(config.clone());

        Ok(Self {
            browser: Arc::new(browser),
            config,
            extractor,
            semaphore,
        })
    }

    /// Create a new Chrome scraper with default configuration
    pub async fn with_defaults() -> Result<Self> {
        Self::new(ScraperConfig::default()).await
    }

    /// Scrape a single page and extract content
    async fn scrape_page(&self, url: &str) -> Result<ScrapeResult> {
        let page = self
            .browser
            .new_page(url)
            .await
            .map_err(|e| RivuletError::Scraper(format!("Failed to create page: {}", e)))?;

        // Set user agent if configured
        if let Some(ref ua) = self.config.user_agent {
            page.set_user_agent(ua)
                .await
                .map_err(|e| RivuletError::Scraper(format!("Failed to set user agent: {}", e)))?;
        }

        // Wait for the page to load
        page.wait_for_navigation()
            .await
            .map_err(|e| RivuletError::Scraper(format!("Navigation failed: {}", e)))?;

        // Additional wait for dynamic content
        tokio::time::sleep(self.config.wait_after_load()).await;

        // Execute extraction script
        let script = self.extractor.extraction_script();
        let result: serde_json::Value = page
            .evaluate(script)
            .await
            .map_err(|e| RivuletError::Scraper(format!("Script execution failed: {}", e)))?
            .into_value()
            .map_err(|e| RivuletError::Scraper(format!("Failed to parse result: {:?}", e)))?;

        // Extract content from result
        let html = result["html"].as_str().unwrap_or("").to_string();
        let text = result["text"].as_str().unwrap_or("").to_string();

        // Close the page
        page.close()
            .await
            .map_err(|e| RivuletError::Scraper(format!("Failed to close page: {}", e)))?;

        // Prefer HTML if available, fallback to text
        if !html.is_empty() {
            Ok(ScrapeResult {
                content: html,
                is_html: true,
            })
        } else if !text.is_empty() {
            Ok(ScrapeResult {
                content: text,
                is_html: false,
            })
        } else {
            Err(RivuletError::Scraper("No content extracted".to_string()))
        }
    }
}

#[async_trait]
impl Scraper for ChromeScraper {
    async fn scrape(&self, url: &str) -> Result<ScrapeResult> {
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|e| RivuletError::Scraper(format!("Semaphore error: {}", e)))?;

        self.scrape_page(url).await
    }

    async fn scrape_items(
        &self,
        items: &[Item],
        concurrency: usize,
    ) -> Vec<(String, Result<ScrapeResult>)> {
        let semaphore = Arc::new(Semaphore::new(concurrency));
        let mut handles = Vec::new();

        for item in items {
            if !Self::needs_scraping(item) {
                continue;
            }

            let Some(ref url) = item.link else {
                continue;
            };

            let item_id = item.id.clone();
            let url = url.clone();
            let sem = semaphore.clone();
            let browser = self.browser.clone();
            let config = self.config.clone();

            let handle = tokio::spawn(async move {
                let _permit = sem.acquire().await;
                let extractor = ContentExtractor::new(config.clone());

                let result = async {
                    let page = browser.new_page(&url).await.map_err(|e| {
                        RivuletError::Scraper(format!("Failed to create page: {}", e))
                    })?;

                    if let Some(ref ua) = config.user_agent {
                        page.set_user_agent(ua).await.map_err(|e| {
                            RivuletError::Scraper(format!("Failed to set user agent: {}", e))
                        })?;
                    }

                    page.wait_for_navigation()
                        .await
                        .map_err(|e| RivuletError::Scraper(format!("Navigation failed: {}", e)))?;

                    tokio::time::sleep(config.wait_after_load()).await;

                    let script = extractor.extraction_script();
                    let result: serde_json::Value = page
                        .evaluate(script)
                        .await
                        .map_err(|e| {
                            RivuletError::Scraper(format!("Script execution failed: {}", e))
                        })?
                        .into_value()
                        .map_err(|e| {
                            RivuletError::Scraper(format!("Failed to parse result: {:?}", e))
                        })?;

                    let html = result["html"].as_str().unwrap_or("").to_string();
                    let text = result["text"].as_str().unwrap_or("").to_string();

                    let _ = page.close().await;

                    if !html.is_empty() {
                        Ok(ScrapeResult {
                            content: html,
                            is_html: true,
                        })
                    } else if !text.is_empty() {
                        Ok(ScrapeResult {
                            content: text,
                            is_html: false,
                        })
                    } else {
                        Err(RivuletError::Scraper("No content extracted".to_string()))
                    }
                }
                .await;

                (item_id, result)
            });

            handles.push(handle);
        }

        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result),
                Err(e) => {
                    tracing::error!("Task join error: {}", e);
                }
            }
        }

        results
    }
}

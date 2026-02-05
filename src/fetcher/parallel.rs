use std::sync::Arc;

use tokio::sync::Semaphore;

use crate::app::Result;
use crate::domain::{Feed, FeedUpdate};
use crate::fetcher::{FetchResult, Fetcher};
use crate::normalizer::Normalizer;
use crate::store::Store;

pub const DEFAULT_WORKERS: usize = 10;

pub struct ParallelFetcher {
    fetcher: Arc<dyn Fetcher + Send + Sync>,
    semaphore: Arc<Semaphore>,
}

impl ParallelFetcher {
    pub fn new(fetcher: Arc<dyn Fetcher + Send + Sync>) -> Self {
        Self::with_workers(fetcher, DEFAULT_WORKERS)
    }

    pub fn with_workers(fetcher: Arc<dyn Fetcher + Send + Sync>, workers: usize) -> Self {
        Self {
            fetcher,
            semaphore: Arc::new(Semaphore::new(workers)),
        }
    }

    pub async fn fetch_all<S: Store + Send + Sync + 'static>(
        &self,
        feeds: Vec<Feed>,
        store: Arc<S>,
        normalizer: &Normalizer,
    ) -> Vec<(i64, Result<usize>)> {
        let mut handles = Vec::new();

        for feed in feeds {
            let fetcher = self.fetcher.clone();
            let semaphore = self.semaphore.clone();
            let store = store.clone();
            let normalizer = normalizer.clone();

            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.expect("Semaphore closed");

                let result = fetch_single_feed(&fetcher, &feed, &store, &normalizer).await;
                (feed.id, result)
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

async fn fetch_single_feed<S: Store>(
    fetcher: &Arc<dyn Fetcher + Send + Sync>,
    feed: &Feed,
    store: &Arc<S>,
    normalizer: &Normalizer,
) -> Result<usize> {
    let result = fetcher
        .fetch(
            &feed.url,
            feed.etag.as_deref(),
            feed.last_modified.as_deref(),
        )
        .await?;

    match result {
        FetchResult::NotModified => {
            tracing::debug!("Feed {} not modified", feed.url);
            Ok(0)
        }
        FetchResult::Content {
            body,
            etag,
            last_modified,
        } => {
            let (feed_meta, items) = normalizer.normalize(feed.id, &feed.url, &body)?;

            // Update feed metadata
            let mut update = FeedUpdate::default();
            if let Some(title) = feed_meta.title {
                update.title = Some(title);
            }
            if let Some(description) = feed_meta.description {
                update.description = Some(description);
            }
            update.etag = etag;
            update.last_modified = last_modified;
            update.last_fetched_at = Some(chrono::Utc::now());

            store.update_feed(feed.id, &update)?;

            // Add new items
            let new_count = store.add_items(&items)?;
            tracing::info!("Added {} new items from {}", new_count, feed.url);

            Ok(new_count)
        }
    }
}

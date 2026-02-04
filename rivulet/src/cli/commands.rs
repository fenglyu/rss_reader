use chrono::Utc;

use crate::app::{AppContext, Result, RivuletError};
use crate::domain::{Feed, FeedUpdate};
use crate::fetcher::FetchResult;
use crate::store::Store;

pub async fn add_feed(ctx: &AppContext, url: &str) -> Result<()> {
    // Check if feed already exists
    if ctx.store.get_feed_by_url(url)?.is_some() {
        println!("Feed already exists: {}", url);
        return Ok(());
    }

    // Create the feed entry
    let feed = Feed::new(url.to_string());
    let feed_id = ctx.store.add_feed(&feed)?;
    println!("Added feed: {}", url);

    // Fetch and store items
    let result = ctx.fetcher.fetch(url, None, None).await?;

    match result {
        FetchResult::Content {
            body,
            etag,
            last_modified,
        } => {
            let (meta, items) = ctx.normalizer.normalize(feed_id, url, &body)?;

            // Update feed with metadata
            let update = FeedUpdate {
                title: meta.title.clone(),
                description: meta.description,
                etag,
                last_modified,
                last_fetched_at: Some(Utc::now()),
            };
            ctx.store.update_feed(feed_id, &update)?;

            // Add items
            let count = ctx.store.add_items(&items)?;

            if let Some(title) = meta.title {
                println!("Feed title: {}", title);
            }
            println!("Fetched {} items", count);
        }
        FetchResult::NotModified => {
            println!("Feed not modified");
        }
    }

    Ok(())
}

pub async fn remove_feed(ctx: &AppContext, url: &str) -> Result<()> {
    let feed = ctx
        .store
        .get_feed_by_url(url)?
        .ok_or_else(|| RivuletError::FeedNotFound(url.to_string()))?;

    ctx.store.delete_feed(feed.id)?;
    println!("Removed feed: {}", url);
    Ok(())
}

pub async fn update_feeds(ctx: &AppContext) -> Result<()> {
    let feeds = ctx.store.get_all_feeds()?;

    if feeds.is_empty() {
        println!("No feeds to update");
        return Ok(());
    }

    println!("Updating {} feeds...", feeds.len());

    let results = ctx
        .parallel_fetcher
        .fetch_all(feeds, ctx.store.clone(), &ctx.normalizer)
        .await;

    let mut total_new = 0;
    let mut errors = 0;

    for (feed_id, result) in results {
        match result {
            Ok(count) => {
                total_new += count;
                if count > 0 {
                    if let Ok(Some(feed)) = ctx.store.get_feed(feed_id) {
                        println!("  {} new items from {}", count, feed.display_title());
                    }
                }
            }
            Err(e) => {
                errors += 1;
                if let Ok(Some(feed)) = ctx.store.get_feed(feed_id) {
                    eprintln!("  Error updating {}: {}", feed.display_title(), e);
                }
            }
        }
    }

    println!("Update complete: {} new items, {} errors", total_new, errors);
    Ok(())
}

pub fn list_feeds(ctx: &AppContext) -> Result<()> {
    let feeds = ctx.store.get_all_feeds()?;

    if feeds.is_empty() {
        println!("No feeds");
        return Ok(());
    }

    for feed in feeds {
        let unread = ctx.store.get_unread_count(feed.id)?;
        println!(
            "{} ({} unread)\n  {}",
            feed.display_title(),
            unread,
            feed.url
        );
    }

    Ok(())
}

pub fn list_items(ctx: &AppContext) -> Result<()> {
    let items = ctx.store.get_all_items()?;

    if items.is_empty() {
        println!("No items");
        return Ok(());
    }

    for item in items {
        let state = ctx.store.get_item_state(&item.id)?;
        let read_marker = if state.map(|s| s.is_read).unwrap_or(false) {
            " "
        } else {
            "●"
        };

        let date = item
            .published_at
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "          ".to_string());

        println!("{} {} {}", read_marker, date, item.display_title());
    }

    Ok(())
}

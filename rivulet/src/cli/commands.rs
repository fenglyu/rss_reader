use std::path::Path;

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

/// Import feeds from an OPML file
pub async fn import_opml(ctx: &AppContext, path: &Path) -> Result<()> {
    let content = std::fs::read_to_string(path)?;
    let feed_urls = parse_opml(&content)?;

    if feed_urls.is_empty() {
        println!("No feeds found in OPML file");
        return Ok(());
    }

    println!("Found {} feeds in OPML file", feed_urls.len());

    let mut added = 0;
    let mut skipped = 0;
    let mut errors = 0;

    for (title, url) in feed_urls {
        // Check if feed already exists
        if ctx.store.get_feed_by_url(&url)?.is_some() {
            skipped += 1;
            continue;
        }

        // Create the feed entry
        let feed = Feed::new(url.clone());
        let feed_id = ctx.store.add_feed(&feed)?;

        // Try to fetch the feed
        match ctx.fetcher.fetch(&url, None, None).await {
            Ok(FetchResult::Content {
                body,
                etag,
                last_modified,
            }) => {
                match ctx.normalizer.normalize(feed_id, &url, &body) {
                    Ok((meta, items)) => {
                        let update = FeedUpdate {
                            title: meta.title.or_else(|| Some(title.clone())),
                            description: meta.description,
                            etag,
                            last_modified,
                            last_fetched_at: Some(Utc::now()),
                        };
                        ctx.store.update_feed(feed_id, &update)?;
                        let count = ctx.store.add_items(&items)?;
                        println!("  + {} ({} items)", title, count);
                        added += 1;
                    }
                    Err(e) => {
                        eprintln!("  ! {} - parse error: {}", title, e);
                        ctx.store.delete_feed(feed_id)?;
                        errors += 1;
                    }
                }
            }
            Ok(FetchResult::NotModified) => {
                println!("  + {} (not modified)", title);
                added += 1;
            }
            Err(e) => {
                eprintln!("  ! {} - fetch error: {}", title, e);
                ctx.store.delete_feed(feed_id)?;
                errors += 1;
            }
        }
    }

    println!(
        "\nImport complete: {} added, {} skipped (already exist), {} errors",
        added, skipped, errors
    );

    Ok(())
}

/// Parse OPML content and extract feed URLs with titles
fn parse_opml(content: &str) -> Result<Vec<(String, String)>> {
    let mut feeds = Vec::new();

    // Simple regex-free parsing: find all outline elements with xmlUrl
    for line in content.lines() {
        if line.contains("xmlUrl") {
            if let (Some(title), Some(url)) = (extract_attr(line, "title")
                .or_else(|| extract_attr(line, "text")), extract_attr(line, "xmlUrl"))
            {
                feeds.push((title, url));
            }
        }
    }

    Ok(feeds)
}

/// Extract an attribute value from an XML element string
fn extract_attr(line: &str, attr: &str) -> Option<String> {
    let pattern = format!("{}=\"", attr);
    let start = line.find(&pattern)? + pattern.len();
    let rest = &line[start..];
    let end = rest.find('"')?;
    let value = &rest[..end];
    Some(html_escape::decode_html_entities(value).to_string())
}

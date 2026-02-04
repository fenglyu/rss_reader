use std::path::Path;

use chrono::Utc;

use crate::app::{AppContext, Result, RivuletError};
use crate::domain::{Feed, FeedUpdate};
use crate::fetcher::FetchResult;
use crate::scraper::{ChromeScraper, Scraper, ScraperConfig};
use crate::store::Store;

/// Initialize config file with all options
pub fn init_config(force: bool) -> Result<()> {
    let config_path = crate::config::Config::default_config_path()
        .map_err(|e| RivuletError::Config(e.to_string()))?;

    if config_path.exists() && !force {
        println!("Config already exists: {}", config_path.display());
        println!("Use --force to overwrite");
        return Ok(());
    }

    // Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Write the full sample config
    let sample_config = include_str!("../../config.sample.toml");
    std::fs::write(&config_path, sample_config)?;

    println!("Created config: {}", config_path.display());
    println!("\nEdit to customize colors, keybindings, and scraper settings.");

    Ok(())
}

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

            // Queue new items for background scraping
            ctx.queue_for_scraping(items).await;

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
        .fetch_all(feeds.clone(), ctx.store.clone(), &ctx.normalizer)
        .await;

    let mut total_new = 0;
    let mut errors = 0;
    let mut updated_feed_ids = Vec::new();

    for (feed_id, result) in results {
        match result {
            Ok(count) => {
                total_new += count;
                if count > 0 {
                    updated_feed_ids.push(feed_id);
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

    // Queue items from updated feeds for background scraping
    if ctx.scraper_handle.is_some() && !updated_feed_ids.is_empty() {
        let mut items_to_scrape = Vec::new();
        for feed_id in updated_feed_ids {
            if let Ok(items) = ctx.store.get_items_by_feed(feed_id) {
                items_to_scrape.extend(items);
            }
        }
        if !items_to_scrape.is_empty() {
            println!("Queuing {} items for background content scraping...", items_to_scrape.len());
            ctx.queue_for_scraping(items_to_scrape).await;
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

    // First pass: filter out existing feeds and create new feed entries
    let mut feeds_to_fetch = Vec::new();
    let mut skipped = 0;

    for (title, url) in feed_urls {
        if ctx.store.get_feed_by_url(&url)?.is_some() {
            skipped += 1;
            continue;
        }

        // Create the feed entry
        let feed = Feed::new(url.clone());
        let feed_id = ctx.store.add_feed(&feed)?;

        // Store with OPML title as fallback
        let mut feed_with_id = feed;
        feed_with_id.id = feed_id;
        feed_with_id.title = Some(title);
        feeds_to_fetch.push(feed_with_id);
    }

    if feeds_to_fetch.is_empty() {
        println!("All feeds already exist ({} skipped)", skipped);
        return Ok(());
    }

    println!("Fetching {} new feeds in parallel...", feeds_to_fetch.len());

    // Parallel fetch all new feeds
    let results = ctx
        .parallel_fetcher
        .fetch_all(feeds_to_fetch.clone(), ctx.store.clone(), &ctx.normalizer)
        .await;

    let mut added = 0;
    let mut errors = 0;

    let mut imported_feed_ids = Vec::new();

    for (feed_id, result) in results {
        let feed = feeds_to_fetch.iter().find(|f| f.id == feed_id);
        let title = feed
            .and_then(|f| f.title.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("Unknown");

        match result {
            Ok(count) => {
                println!("  + {} ({} items)", title, count);
                added += 1;
                imported_feed_ids.push(feed_id);
            }
            Err(e) => {
                eprintln!("  ! {} - error: {}", title, e);
                // Delete the feed entry on error
                let _ = ctx.store.delete_feed(feed_id);
                errors += 1;
            }
        }
    }

    // Queue items from imported feeds for background scraping
    if ctx.scraper_handle.is_some() && !imported_feed_ids.is_empty() {
        let mut items_to_scrape = Vec::new();
        for feed_id in imported_feed_ids {
            if let Ok(items) = ctx.store.get_items_by_feed(feed_id) {
                items_to_scrape.extend(items);
            }
        }
        if !items_to_scrape.is_empty() {
            println!("Queuing {} items for background content scraping...", items_to_scrape.len());
            ctx.queue_for_scraping(items_to_scrape).await;
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

/// Scrape full content for items that only have summaries
pub async fn scrape_content(
    ctx: &AppContext,
    feed_url: Option<&str>,
    limit: usize,
    concurrency: usize,
    visible: bool,
) -> Result<()> {
    // Get items to scrape
    let items = if let Some(url) = feed_url {
        let feed = ctx
            .store
            .get_feed_by_url(url)?
            .ok_or_else(|| RivuletError::FeedNotFound(url.to_string()))?;
        ctx.store.get_items_by_feed(feed.id)?
    } else {
        ctx.store.get_all_items()?
    };

    // Filter items that need scraping (have link but no/short content)
    let items_to_scrape: Vec<_> = items
        .into_iter()
        .filter(|item| ChromeScraper::needs_scraping(item))
        .take(limit)
        .collect();

    if items_to_scrape.is_empty() {
        println!("No items need scraping");
        return Ok(());
    }

    println!(
        "Scraping {} items with {} concurrent pages...",
        items_to_scrape.len(),
        concurrency
    );

    // Create scraper config
    let config = ScraperConfig {
        headless: !visible,
        max_concurrency: concurrency,
        ..Default::default()
    };

    // Initialize the scraper
    let scraper = ChromeScraper::new(config).await?;

    // Scrape all items
    let results = scraper.scrape_items(&items_to_scrape, concurrency).await;

    let mut success = 0;
    let mut errors = 0;

    for (item_id, result) in results {
        let item = items_to_scrape.iter().find(|i| i.id == item_id);
        let title = item.map(|i| i.display_title()).unwrap_or("Unknown");

        match result {
            Ok(scrape_result) => {
                // Update item content in store
                ctx.store.update_item_content(&item_id, &scrape_result.content)?;
                let content_type = if scrape_result.is_html { "HTML" } else { "text" };
                println!(
                    "  + {} ({}, {} chars)",
                    title,
                    content_type,
                    scrape_result.content.len()
                );
                success += 1;
            }
            Err(e) => {
                eprintln!("  ! {} - error: {}", title, e);
                errors += 1;
            }
        }
    }

    println!(
        "\nScraping complete: {} succeeded, {} failed",
        success, errors
    );

    Ok(())
}

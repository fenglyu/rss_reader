# Web Scraper Module

Rivulet includes a built-in web scraper that fetches full article content from web pages when RSS feeds only provide summaries or metadata.

## Overview

Many RSS feeds (like `beej.us/blog/rss.xml`) only include titles, dates, and brief descriptions without the full article content. The scraper module uses headless Chrome to:

1. Navigate to article URLs
2. Wait for dynamic content to load
3. Extract the main article content
4. Store it in the database for offline reading

## Features

- **Headless Chrome** - Uses `chromiumoxide` for browser automation
- **Background Processing** - Non-blocking, runs in a separate tokio task
- **Automatic Triggering** - Scrapes new items during add/import/update operations
- **Configurable Selectors** - Customize content extraction for different sites
- **Concurrent Scraping** - Process multiple pages in parallel

## Usage

### Automatic Background Scraping

When enabled in config, scraping happens automatically:

```bash
# Adding a feed triggers scraping for new items
rivulet add https://beej.us/blog/rss.xml

# Updating feeds triggers scraping for new items
rivulet update

# Importing OPML triggers scraping for all imported items
rivulet import feeds.opml
```

### Manual Scraping

```bash
# Scrape items (default: 10 items, 3 concurrent)
rivulet scrape

# Scrape specific feed
rivulet scrape --feed "https://beej.us/blog/rss.xml"

# Scrape more items with higher concurrency
rivulet scrape --limit 50 --concurrency 5

# Debug mode (show browser window)
rivulet scrape --visible
```

### TUI Integration

Press `R` in the TUI to refresh feeds - new items are automatically queued for background scraping.

## Configuration

Add to `~/.config/rivulet/config.toml`:

```toml
[scraper]
# Enable/disable automatic background scraping
enabled = true

# Run browser in headless mode (no visible window)
headless = true

# Minimum content length to consider an item as having content
# Items with content shorter than this will be scraped
min_content_length = 200

# Page load timeout in seconds
timeout_secs = 30

# Wait time after page load for dynamic content (milliseconds)
wait_after_load_ms = 1000

# Maximum concurrent browser pages
max_concurrency = 3

# Block images for faster loading
block_images = true

# Block stylesheets for faster loading
block_stylesheets = true

# CSS selectors to try for article content extraction (in priority order)
content_selectors = [
    "article",
    "[role=\"main\"]",
    "main",
    ".post-content",
    ".article-content",
    ".entry-content",
    ".content",
    "#content",
    ".post",
    ".article",
]

# Elements to remove before extraction (ads, navigation, etc.)
remove_selectors = [
    "nav",
    "header",
    "footer",
    "aside",
    ".sidebar",
    ".advertisement",
    ".ad",
    ".ads",
    ".social-share",
    ".comments",
    "script",
    "style",
]

# Optional: Custom user agent string
# user_agent = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) ..."
```

## How It Works

### Content Extraction Flow

```
1. Item has link but no/short content
         ↓
2. Queue item for background scraping
         ↓
3. Launch headless Chrome page
         ↓
4. Navigate to article URL
         ↓
5. Wait for page load + dynamic content
         ↓
6. Remove unwanted elements (ads, nav, etc.)
         ↓
7. Try content selectors in order
         ↓
8. Extract innerHTML from first match
         ↓
9. Store content in database
```

### Selector Priority

The scraper tries `content_selectors` in order and uses the first one that:
- Matches an element on the page
- Contains more than 100 characters of text

If none match, it falls back to `<body>`.

### Resource Blocking

When `block_images` and `block_stylesheets` are enabled, the scraper:
- Skips loading images (faster page loads)
- Skips loading CSS files
- Skips loading fonts

This significantly speeds up scraping but may affect some JavaScript-heavy sites.

## Architecture

### Components

```
src/scraper/
├── mod.rs          # Module exports, Scraper trait
├── chrome.rs       # ChromeScraper implementation
├── config.rs       # ScraperConfig struct
├── extractor.rs    # JavaScript extraction scripts
└── background.rs   # Background scraper service
```

### Background Scraper

The `BackgroundScraper` runs as a tokio task:

```rust
// Spawned when AppContext is created with scraper enabled
let handle = spawn_background_scraper(config, store);

// Queue items for scraping (non-blocking)
handle.queue_items(items).await;

// Graceful shutdown
handle.shutdown().await;
```

### Integration Points

1. **AppContext** - Holds optional `BackgroundScraperHandle`
2. **CLI Commands** - Queue items after add/import/update
3. **TUI Refresh** - Queue items after feed refresh
4. **Manual Command** - Direct scraping via `rivulet scrape`

## Troubleshooting

### Chrome Not Found

The scraper requires Chrome/Chromium to be installed:

```bash
# macOS
brew install --cask google-chrome

# Ubuntu/Debian
sudo apt install chromium-browser

# Fedora
sudo dnf install chromium
```

### Deserialization Warnings

You may see warnings like:
```
Failed to deserialize WS response data did not match any variant
```

These are harmless - they're caused by new Chrome DevTools Protocol messages that the `chromiumoxide` library doesn't yet recognize.

### Content Not Extracted

If content isn't being extracted properly:

1. **Check selectors** - The site may use different selectors
2. **Increase wait time** - Dynamic content may need more time
3. **Disable resource blocking** - Some sites require CSS/JS

Add site-specific selectors to `content_selectors`:

```toml
content_selectors = [
    ".custom-article-class",  # Site-specific
    "article",
    "main",
    # ... default selectors
]
```

### Memory Usage

Each browser page uses memory. Control this with:

```toml
max_concurrency = 3  # Reduce for lower memory usage
```

### Scraping Takes Too Long

Speed up scraping:

```toml
timeout_secs = 15           # Reduce timeout
wait_after_load_ms = 500    # Reduce wait time
block_images = true         # Skip images
block_stylesheets = true    # Skip CSS
max_concurrency = 5         # More parallel pages
```

## API Reference

### Scraper Trait

```rust
#[async_trait]
pub trait Scraper: Send + Sync {
    /// Scrape content from a URL
    async fn scrape(&self, url: &str) -> Result<ScrapeResult>;

    /// Scrape content for multiple items concurrently
    async fn scrape_items(
        &self,
        items: &[Item],
        concurrency: usize,
    ) -> Vec<(String, Result<ScrapeResult>)>;

    /// Check if an item needs content scraping
    fn needs_scraping(item: &Item) -> bool;
}
```

### ScrapeResult

```rust
pub struct ScrapeResult {
    /// The extracted article content (HTML or plain text)
    pub content: String,
    /// Whether the content is HTML (true) or plain text (false)
    pub is_html: bool,
}
```

### ScraperConfig

```rust
pub struct ScraperConfig {
    pub enabled: bool,
    pub headless: bool,
    pub min_content_length: usize,
    pub timeout_secs: u64,
    pub wait_after_load_ms: u64,
    pub content_selectors: Vec<String>,
    pub remove_selectors: Vec<String>,
    pub max_concurrency: usize,
    pub block_images: bool,
    pub block_stylesheets: bool,
    pub user_agent: Option<String>,
}
```

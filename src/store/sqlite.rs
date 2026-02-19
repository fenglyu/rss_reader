use std::path::Path;
use std::sync::Mutex;

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use rusqlite_migration::{Migrations, M};

use crate::app::{Result, RivuletError};
use crate::domain::{Feed, FeedUpdate, Item, ItemState};
use crate::store::Store;

pub struct SqliteStore {
    conn: Mutex<Connection>,
}

impl SqliteStore {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        let store = Self {
            conn: Mutex::new(conn),
        };
        store.run_migrations()?;
        Ok(store)
    }

    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let store = Self {
            conn: Mutex::new(conn),
        };
        store.run_migrations()?;
        Ok(store)
    }

    fn run_migrations(&self) -> Result<()> {
        let migrations = Migrations::new(vec![M::up(include_str!(
            "../../migrations/001-initial/up.sql"
        ))]);

        let mut conn = self.conn.lock().map_err(|e| {
            RivuletError::Database(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(1),
                Some(e.to_string()),
            ))
        })?;

        conn.execute("PRAGMA foreign_keys = ON", [])?;
        migrations
            .to_latest(&mut conn)
            .map_err(|_| RivuletError::Database(rusqlite::Error::InvalidQuery))?;

        Ok(())
    }

    fn parse_datetime(s: &str) -> Option<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(s)
            .map(|dt| dt.with_timezone(&Utc))
            .ok()
            .or_else(|| s.parse::<DateTime<Utc>>().ok())
    }
}

impl Store for SqliteStore {
    fn add_feed(&self, feed: &Feed) -> Result<i64> {
        let conn = self.conn.lock().map_err(|e| {
            RivuletError::Database(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(1),
                Some(e.to_string()),
            ))
        })?;

        conn.execute(
            "INSERT INTO feeds (url, title, description, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![
                feed.url,
                feed.title,
                feed.description,
                feed.created_at.to_rfc3339()
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    fn get_feed(&self, id: i64) -> Result<Option<Feed>> {
        let conn = self.conn.lock().map_err(|e| {
            RivuletError::Database(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(1),
                Some(e.to_string()),
            ))
        })?;

        let result = conn
            .query_row(
                "SELECT id, url, title, description, etag, last_modified, last_fetched_at, created_at
                 FROM feeds WHERE id = ?1",
                params![id],
                |row| {
                    Ok(Feed {
                        id: row.get(0)?,
                        url: row.get(1)?,
                        title: row.get(2)?,
                        description: row.get(3)?,
                        etag: row.get(4)?,
                        last_modified: row.get(5)?,
                        last_fetched_at: row.get::<_, Option<String>>(6)?
                            .and_then(|s| Self::parse_datetime(&s)),
                        created_at: row.get::<_, String>(7)
                            .ok()
                            .and_then(|s| Self::parse_datetime(&s))
                            .unwrap_or_else(Utc::now),
                    })
                },
            )
            .optional()?;

        Ok(result)
    }

    fn get_feed_by_url(&self, url: &str) -> Result<Option<Feed>> {
        let conn = self.conn.lock().map_err(|e| {
            RivuletError::Database(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(1),
                Some(e.to_string()),
            ))
        })?;

        let result = conn
            .query_row(
                "SELECT id, url, title, description, etag, last_modified, last_fetched_at, created_at
                 FROM feeds WHERE url = ?1",
                params![url],
                |row| {
                    Ok(Feed {
                        id: row.get(0)?,
                        url: row.get(1)?,
                        title: row.get(2)?,
                        description: row.get(3)?,
                        etag: row.get(4)?,
                        last_modified: row.get(5)?,
                        last_fetched_at: row.get::<_, Option<String>>(6)?
                            .and_then(|s| Self::parse_datetime(&s)),
                        created_at: row.get::<_, String>(7)
                            .ok()
                            .and_then(|s| Self::parse_datetime(&s))
                            .unwrap_or_else(Utc::now),
                    })
                },
            )
            .optional()?;

        Ok(result)
    }

    fn get_all_feeds(&self) -> Result<Vec<Feed>> {
        let conn = self.conn.lock().map_err(|e| {
            RivuletError::Database(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(1),
                Some(e.to_string()),
            ))
        })?;

        let mut stmt = conn.prepare(
            "SELECT id, url, title, description, etag, last_modified, last_fetched_at, created_at
             FROM feeds ORDER BY title, url",
        )?;

        let feeds = stmt
            .query_map([], |row| {
                Ok(Feed {
                    id: row.get(0)?,
                    url: row.get(1)?,
                    title: row.get(2)?,
                    description: row.get(3)?,
                    etag: row.get(4)?,
                    last_modified: row.get(5)?,
                    last_fetched_at: row
                        .get::<_, Option<String>>(6)?
                        .and_then(|s| Self::parse_datetime(&s)),
                    created_at: row
                        .get::<_, String>(7)
                        .ok()
                        .and_then(|s| Self::parse_datetime(&s))
                        .unwrap_or_else(Utc::now),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(feeds)
    }

    fn update_feed(&self, id: i64, update: &FeedUpdate) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| {
            RivuletError::Database(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(1),
                Some(e.to_string()),
            ))
        })?;

        if let Some(ref title) = update.title {
            conn.execute(
                "UPDATE feeds SET title = ?1 WHERE id = ?2",
                params![title, id],
            )?;
        }
        if let Some(ref description) = update.description {
            conn.execute(
                "UPDATE feeds SET description = ?1 WHERE id = ?2",
                params![description, id],
            )?;
        }
        if let Some(ref etag) = update.etag {
            conn.execute(
                "UPDATE feeds SET etag = ?1 WHERE id = ?2",
                params![etag, id],
            )?;
        }
        if let Some(ref last_modified) = update.last_modified {
            conn.execute(
                "UPDATE feeds SET last_modified = ?1 WHERE id = ?2",
                params![last_modified, id],
            )?;
        }
        if let Some(ref last_fetched_at) = update.last_fetched_at {
            conn.execute(
                "UPDATE feeds SET last_fetched_at = ?1 WHERE id = ?2",
                params![last_fetched_at.to_rfc3339(), id],
            )?;
        }

        Ok(())
    }

    fn delete_feed(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| {
            RivuletError::Database(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(1),
                Some(e.to_string()),
            ))
        })?;

        conn.execute("DELETE FROM feeds WHERE id = ?1", params![id])?;
        Ok(())
    }

    fn add_item(&self, item: &Item) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| {
            RivuletError::Database(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(1),
                Some(e.to_string()),
            ))
        })?;

        conn.execute(
            "INSERT OR IGNORE INTO items (id, feed_id, title, link, content, summary, author, published_at, fetched_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                item.id,
                item.feed_id,
                item.title,
                item.link,
                item.content,
                item.summary,
                item.author,
                item.published_at.map(|dt| dt.to_rfc3339()),
                item.fetched_at.to_rfc3339()
            ],
        )?;

        Ok(())
    }

    fn add_items(&self, items: &[Item]) -> Result<usize> {
        let mut conn = self.conn.lock().map_err(|e| {
            RivuletError::Database(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(1),
                Some(e.to_string()),
            ))
        })?;

        let tx = conn.transaction()?;
        let mut count = 0;

        for item in items {
            let inserted = tx.execute(
                "INSERT OR IGNORE INTO items (id, feed_id, title, link, content, summary, author, published_at, fetched_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    item.id,
                    item.feed_id,
                    item.title,
                    item.link,
                    item.content,
                    item.summary,
                    item.author,
                    item.published_at.map(|dt| dt.to_rfc3339()),
                    item.fetched_at.to_rfc3339()
                ],
            )?;
            count += inserted;
        }

        tx.commit()?;
        Ok(count)
    }

    fn get_item(&self, id: &str) -> Result<Option<Item>> {
        let conn = self.conn.lock().map_err(|e| {
            RivuletError::Database(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(1),
                Some(e.to_string()),
            ))
        })?;

        let result = conn
            .query_row(
                "SELECT id, feed_id, title, link, content, summary, author, published_at, fetched_at
                 FROM items WHERE id = ?1",
                params![id],
                |row| {
                    Ok(Item {
                        id: row.get(0)?,
                        feed_id: row.get(1)?,
                        title: row.get(2)?,
                        link: row.get(3)?,
                        content: row.get(4)?,
                        summary: row.get(5)?,
                        author: row.get(6)?,
                        published_at: row.get::<_, Option<String>>(7)?
                            .and_then(|s| Self::parse_datetime(&s)),
                        fetched_at: row.get::<_, String>(8)
                            .ok()
                            .and_then(|s| Self::parse_datetime(&s))
                            .unwrap_or_else(Utc::now),
                    })
                },
            )
            .optional()?;

        Ok(result)
    }

    fn get_items_by_feed(&self, feed_id: i64) -> Result<Vec<Item>> {
        let conn = self.conn.lock().map_err(|e| {
            RivuletError::Database(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(1),
                Some(e.to_string()),
            ))
        })?;

        let mut stmt = conn.prepare(
            "SELECT id, feed_id, title, link, content, summary, author, published_at, fetched_at
             FROM items WHERE feed_id = ?1 ORDER BY published_at DESC, fetched_at DESC",
        )?;

        let items = stmt
            .query_map(params![feed_id], |row| {
                Ok(Item {
                    id: row.get(0)?,
                    feed_id: row.get(1)?,
                    title: row.get(2)?,
                    link: row.get(3)?,
                    content: row.get(4)?,
                    summary: row.get(5)?,
                    author: row.get(6)?,
                    published_at: row
                        .get::<_, Option<String>>(7)?
                        .and_then(|s| Self::parse_datetime(&s)),
                    fetched_at: row
                        .get::<_, String>(8)
                        .ok()
                        .and_then(|s| Self::parse_datetime(&s))
                        .unwrap_or_else(Utc::now),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(items)
    }

    fn get_all_items(&self) -> Result<Vec<Item>> {
        let conn = self.conn.lock().map_err(|e| {
            RivuletError::Database(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(1),
                Some(e.to_string()),
            ))
        })?;

        let mut stmt = conn.prepare(
            "SELECT id, feed_id, title, link, content, summary, author, published_at, fetched_at
             FROM items ORDER BY published_at DESC, fetched_at DESC",
        )?;

        let items = stmt
            .query_map([], |row| {
                Ok(Item {
                    id: row.get(0)?,
                    feed_id: row.get(1)?,
                    title: row.get(2)?,
                    link: row.get(3)?,
                    content: row.get(4)?,
                    summary: row.get(5)?,
                    author: row.get(6)?,
                    published_at: row
                        .get::<_, Option<String>>(7)?
                        .and_then(|s| Self::parse_datetime(&s)),
                    fetched_at: row
                        .get::<_, String>(8)
                        .ok()
                        .and_then(|s| Self::parse_datetime(&s))
                        .unwrap_or_else(Utc::now),
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(items)
    }

    fn item_exists(&self, id: &str) -> Result<bool> {
        let conn = self.conn.lock().map_err(|e| {
            RivuletError::Database(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(1),
                Some(e.to_string()),
            ))
        })?;

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM items WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )?;

        Ok(count > 0)
    }

    fn get_item_state(&self, item_id: &str) -> Result<Option<ItemState>> {
        let conn = self.conn.lock().map_err(|e| {
            RivuletError::Database(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(1),
                Some(e.to_string()),
            ))
        })?;

        let result = conn
            .query_row(
                "SELECT item_id, is_read, is_starred, read_at, starred_at
                 FROM item_state WHERE item_id = ?1",
                params![item_id],
                |row| {
                    Ok(ItemState {
                        item_id: row.get(0)?,
                        is_read: row.get::<_, i32>(1)? != 0,
                        is_starred: row.get::<_, i32>(2)? != 0,
                        read_at: row
                            .get::<_, Option<String>>(3)?
                            .and_then(|s| Self::parse_datetime(&s)),
                        starred_at: row
                            .get::<_, Option<String>>(4)?
                            .and_then(|s| Self::parse_datetime(&s)),
                    })
                },
            )
            .optional()?;

        Ok(result)
    }

    fn set_read(&self, item_id: &str, is_read: bool) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| {
            RivuletError::Database(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(1),
                Some(e.to_string()),
            ))
        })?;

        let read_at = if is_read {
            Some(Utc::now().to_rfc3339())
        } else {
            None
        };

        conn.execute(
            "INSERT INTO item_state (item_id, is_read, read_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(item_id) DO UPDATE SET is_read = ?2, read_at = ?3",
            params![item_id, is_read as i32, read_at],
        )?;

        Ok(())
    }

    fn set_starred(&self, item_id: &str, is_starred: bool) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| {
            RivuletError::Database(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(1),
                Some(e.to_string()),
            ))
        })?;

        let starred_at = if is_starred {
            Some(Utc::now().to_rfc3339())
        } else {
            None
        };

        conn.execute(
            "INSERT INTO item_state (item_id, is_starred, starred_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(item_id) DO UPDATE SET is_starred = ?2, starred_at = ?3",
            params![item_id, is_starred as i32, starred_at],
        )?;

        Ok(())
    }

    fn get_unread_count(&self, feed_id: i64) -> Result<i64> {
        let conn = self.conn.lock().map_err(|e| {
            RivuletError::Database(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(1),
                Some(e.to_string()),
            ))
        })?;

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM items i
             LEFT JOIN item_state s ON i.id = s.item_id
             WHERE i.feed_id = ?1 AND (s.is_read IS NULL OR s.is_read = 0)",
            params![feed_id],
            |row| row.get(0),
        )?;

        Ok(count)
    }

    fn update_item_content(&self, id: &str, content: &str) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| {
            RivuletError::Database(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(1),
                Some(e.to_string()),
            ))
        })?;

        conn.execute(
            "UPDATE items SET content = ?1 WHERE id = ?2",
            params![content, id],
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_get_feed() {
        let store = SqliteStore::in_memory().unwrap();
        let feed = Feed::new("https://example.com/feed.xml".into());
        let id = store.add_feed(&feed).unwrap();

        let retrieved = store.get_feed(id).unwrap().unwrap();
        assert_eq!(retrieved.url, "https://example.com/feed.xml");
    }

    #[test]
    fn test_add_and_get_item() {
        let store = SqliteStore::in_memory().unwrap();
        let feed = Feed::new("https://example.com/feed.xml".into());
        let feed_id = store.add_feed(&feed).unwrap();

        let mut item = Item::new(feed_id, "https://example.com/feed.xml", "entry-1");
        item.title = Some("Test Item".into());
        store.add_item(&item).unwrap();

        let retrieved = store.get_item(&item.id).unwrap().unwrap();
        assert_eq!(retrieved.title, Some("Test Item".into()));
    }

    #[test]
    fn test_set_read_state() {
        let store = SqliteStore::in_memory().unwrap();
        let feed = Feed::new("https://example.com/feed.xml".into());
        let feed_id = store.add_feed(&feed).unwrap();

        let item = Item::new(feed_id, "https://example.com/feed.xml", "entry-1");
        store.add_item(&item).unwrap();

        store.set_read(&item.id, true).unwrap();
        let state = store.get_item_state(&item.id).unwrap().unwrap();
        assert!(state.is_read);

        store.set_read(&item.id, false).unwrap();
        let state = store.get_item_state(&item.id).unwrap().unwrap();
        assert!(!state.is_read);
    }

    #[test]
    fn test_unread_count() {
        let store = SqliteStore::in_memory().unwrap();
        let feed = Feed::new("https://example.com/feed.xml".into());
        let feed_id = store.add_feed(&feed).unwrap();

        for i in 0..5 {
            let item = Item::new(
                feed_id,
                "https://example.com/feed.xml",
                &format!("entry-{}", i),
            );
            store.add_item(&item).unwrap();
        }

        assert_eq!(store.get_unread_count(feed_id).unwrap(), 5);

        let items = store.get_items_by_feed(feed_id).unwrap();
        store.set_read(&items[0].id, true).unwrap();
        store.set_read(&items[1].id, true).unwrap();

        assert_eq!(store.get_unread_count(feed_id).unwrap(), 3);
    }

    #[test]
    fn test_get_feed_by_url() {
        let store = SqliteStore::in_memory().unwrap();
        let feed = Feed::new("https://example.com/feed.xml".into());
        store.add_feed(&feed).unwrap();

        let found = store
            .get_feed_by_url("https://example.com/feed.xml")
            .unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().url, "https://example.com/feed.xml");

        let missing = store
            .get_feed_by_url("https://example.com/nonexistent.xml")
            .unwrap();
        assert!(missing.is_none());
    }

    #[test]
    fn test_get_all_feeds_ordering() {
        let store = SqliteStore::in_memory().unwrap();

        let feed_c = Feed::new("https://example.com/c.xml".into());
        let feed_a = Feed::new("https://example.com/a.xml".into());
        let feed_b = Feed::new("https://example.com/b.xml".into());

        store.add_feed(&feed_c).unwrap();
        store.add_feed(&feed_a).unwrap();
        store.add_feed(&feed_b).unwrap();

        // With no titles set, should be ordered by URL
        let feeds = store.get_all_feeds().unwrap();
        assert_eq!(feeds.len(), 3);
        assert_eq!(feeds[0].url, "https://example.com/a.xml");
        assert_eq!(feeds[1].url, "https://example.com/b.xml");
        assert_eq!(feeds[2].url, "https://example.com/c.xml");
    }

    #[test]
    fn test_update_feed() {
        let store = SqliteStore::in_memory().unwrap();
        let feed = Feed::new("https://example.com/feed.xml".into());
        let id = store.add_feed(&feed).unwrap();

        let update = FeedUpdate {
            title: Some("Updated Title".into()),
            description: Some("Updated Description".into()),
            etag: Some("\"abc123\"".into()),
            last_modified: Some("Mon, 01 Jan 2024 00:00:00 GMT".into()),
            last_fetched_at: None,
        };
        store.update_feed(id, &update).unwrap();

        let retrieved = store.get_feed(id).unwrap().unwrap();
        assert_eq!(retrieved.title, Some("Updated Title".into()));
        assert_eq!(retrieved.description, Some("Updated Description".into()));
        assert_eq!(retrieved.etag, Some("\"abc123\"".into()));
        assert_eq!(
            retrieved.last_modified,
            Some("Mon, 01 Jan 2024 00:00:00 GMT".into())
        );
    }

    #[test]
    fn test_update_feed_partial() {
        let store = SqliteStore::in_memory().unwrap();
        let feed = Feed::new("https://example.com/feed.xml".into());
        let id = store.add_feed(&feed).unwrap();

        // Only update title, leave everything else as None
        let update = FeedUpdate {
            title: Some("New Title".into()),
            ..Default::default()
        };
        store.update_feed(id, &update).unwrap();

        let retrieved = store.get_feed(id).unwrap().unwrap();
        assert_eq!(retrieved.title, Some("New Title".into()));
        assert_eq!(retrieved.description, None);
        assert_eq!(retrieved.etag, None);
    }

    #[test]
    fn test_delete_feed_cascades_items() {
        let store = SqliteStore::in_memory().unwrap();
        let feed = Feed::new("https://example.com/feed.xml".into());
        let feed_id = store.add_feed(&feed).unwrap();

        let item = Item::new(feed_id, "https://example.com/feed.xml", "entry-1");
        let item_id = item.id.clone();
        store.add_item(&item).unwrap();

        assert!(store.item_exists(&item_id).unwrap());

        store.delete_feed(feed_id).unwrap();

        assert!(store.get_feed(feed_id).unwrap().is_none());
        assert!(!store.item_exists(&item_id).unwrap());
    }

    #[test]
    fn test_add_items_batch_and_dedup() {
        let store = SqliteStore::in_memory().unwrap();
        let feed = Feed::new("https://example.com/feed.xml".into());
        let feed_id = store.add_feed(&feed).unwrap();

        let items: Vec<Item> = (0..3)
            .map(|i| {
                Item::new(
                    feed_id,
                    "https://example.com/feed.xml",
                    &format!("entry-{}", i),
                )
            })
            .collect();

        let count = store.add_items(&items).unwrap();
        assert_eq!(count, 3);

        // Duplicate batch: INSERT OR IGNORE means 0 new rows
        let count = store.add_items(&items).unwrap();
        assert_eq!(count, 0);

        let stored = store.get_items_by_feed(feed_id).unwrap();
        assert_eq!(stored.len(), 3);
    }

    #[test]
    fn test_add_duplicate_item_ignored() {
        let store = SqliteStore::in_memory().unwrap();
        let feed = Feed::new("https://example.com/feed.xml".into());
        let feed_id = store.add_feed(&feed).unwrap();

        let mut item = Item::new(feed_id, "https://example.com/feed.xml", "entry-1");
        item.title = Some("Original Title".into());
        store.add_item(&item).unwrap();

        // Same ID with different title — should be ignored
        let mut dup = Item::new(feed_id, "https://example.com/feed.xml", "entry-1");
        dup.title = Some("Different Title".into());
        store.add_item(&dup).unwrap();

        let retrieved = store.get_item(&item.id).unwrap().unwrap();
        assert_eq!(retrieved.title, Some("Original Title".into()));
    }

    #[test]
    fn test_item_exists() {
        let store = SqliteStore::in_memory().unwrap();
        let feed = Feed::new("https://example.com/feed.xml".into());
        let feed_id = store.add_feed(&feed).unwrap();

        let item = Item::new(feed_id, "https://example.com/feed.xml", "entry-1");
        let item_id = item.id.clone();

        assert!(!store.item_exists(&item_id).unwrap());
        store.add_item(&item).unwrap();
        assert!(store.item_exists(&item_id).unwrap());
    }

    #[test]
    fn test_set_starred() {
        let store = SqliteStore::in_memory().unwrap();
        let feed = Feed::new("https://example.com/feed.xml".into());
        let feed_id = store.add_feed(&feed).unwrap();

        let item = Item::new(feed_id, "https://example.com/feed.xml", "entry-1");
        store.add_item(&item).unwrap();

        store.set_starred(&item.id, true).unwrap();
        let state = store.get_item_state(&item.id).unwrap().unwrap();
        assert!(state.is_starred);
        assert!(state.starred_at.is_some());

        store.set_starred(&item.id, false).unwrap();
        let state = store.get_item_state(&item.id).unwrap().unwrap();
        assert!(!state.is_starred);
        assert!(state.starred_at.is_none());
    }

    #[test]
    fn test_update_item_content() {
        let store = SqliteStore::in_memory().unwrap();
        let feed = Feed::new("https://example.com/feed.xml".into());
        let feed_id = store.add_feed(&feed).unwrap();

        let item = Item::new(feed_id, "https://example.com/feed.xml", "entry-1");
        let item_id = item.id.clone();
        store.add_item(&item).unwrap();

        let retrieved = store.get_item(&item_id).unwrap().unwrap();
        assert_eq!(retrieved.content, None);

        store
            .update_item_content(&item_id, "<p>Full article content</p>")
            .unwrap();

        let retrieved = store.get_item(&item_id).unwrap().unwrap();
        assert_eq!(
            retrieved.content,
            Some("<p>Full article content</p>".into())
        );
    }

    #[test]
    fn test_get_all_items_across_feeds() {
        let store = SqliteStore::in_memory().unwrap();

        let feed1 = Feed::new("https://example.com/feed1.xml".into());
        let feed1_id = store.add_feed(&feed1).unwrap();
        let feed2 = Feed::new("https://example.com/feed2.xml".into());
        let feed2_id = store.add_feed(&feed2).unwrap();

        store
            .add_item(&Item::new(
                feed1_id,
                "https://example.com/feed1.xml",
                "e1",
            ))
            .unwrap();
        store
            .add_item(&Item::new(
                feed2_id,
                "https://example.com/feed2.xml",
                "e2",
            ))
            .unwrap();

        let all = store.get_all_items().unwrap();
        assert_eq!(all.len(), 2);

        let feed_ids: Vec<i64> = all.iter().map(|i| i.feed_id).collect();
        assert!(feed_ids.contains(&feed1_id));
        assert!(feed_ids.contains(&feed2_id));
    }

    #[test]
    fn test_get_feed_nonexistent() {
        let store = SqliteStore::in_memory().unwrap();
        assert!(store.get_feed(999).unwrap().is_none());
    }

    #[test]
    fn test_get_item_nonexistent() {
        let store = SqliteStore::in_memory().unwrap();
        assert!(store.get_item("nonexistent-id").unwrap().is_none());
    }

    #[test]
    fn test_get_item_state_nonexistent() {
        let store = SqliteStore::in_memory().unwrap();
        assert!(store.get_item_state("nonexistent-id").unwrap().is_none());
    }
}

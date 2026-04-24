pub mod sqlite;

use crate::app::Result;
use crate::domain::{AuthProfile, Feed, FeedUpdate, Item, ItemState};

pub use sqlite::SqliteStore;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemListFilter {
    All,
    Unread,
    Starred,
    Queued,
    Saved,
    Archived,
}

pub trait Store {
    // Feed operations
    fn add_feed(&self, feed: &Feed) -> Result<i64>;
    fn get_feed(&self, id: i64) -> Result<Option<Feed>>;
    fn get_feed_by_url(&self, url: &str) -> Result<Option<Feed>>;
    fn get_all_feeds(&self) -> Result<Vec<Feed>>;
    fn update_feed(&self, id: i64, update: &FeedUpdate) -> Result<()>;
    fn delete_feed(&self, id: i64) -> Result<()>;

    // Auth profile operations
    fn add_auth_profile(&self, profile: &AuthProfile) -> Result<i64>;
    fn get_auth_profile_by_name(&self, name: &str) -> Result<Option<AuthProfile>>;
    fn get_all_auth_profiles(&self) -> Result<Vec<AuthProfile>>;
    fn update_auth_profile_status(&self, id: i64, status: &str) -> Result<()>;

    // Item operations
    fn add_item(&self, item: &Item) -> Result<()>;
    fn add_items(&self, items: &[Item]) -> Result<usize>;
    fn get_item(&self, id: &str) -> Result<Option<Item>>;
    fn get_items_by_feed(&self, feed_id: i64) -> Result<Vec<Item>>;
    fn get_all_items(&self) -> Result<Vec<Item>>;
    fn get_items_by_filter(&self, filter: ItemListFilter) -> Result<Vec<Item>>;
    fn search_items(&self, query: &str, filter: ItemListFilter, limit: usize) -> Result<Vec<Item>>;
    fn item_exists(&self, id: &str) -> Result<bool>;
    fn update_item_content(&self, id: &str, content: &str) -> Result<()>;

    // State operations
    fn get_item_state(&self, item_id: &str) -> Result<Option<ItemState>>;
    fn set_read(&self, item_id: &str, is_read: bool) -> Result<()>;
    fn set_starred(&self, item_id: &str, is_starred: bool) -> Result<()>;
    fn set_queued(&self, item_id: &str, is_queued: bool) -> Result<()>;
    fn set_saved(&self, item_id: &str, is_saved: bool) -> Result<()>;
    fn set_archived(&self, item_id: &str, is_archived: bool) -> Result<()>;
    fn get_unread_count(&self, feed_id: i64) -> Result<i64>;
}

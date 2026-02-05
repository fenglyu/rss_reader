use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ItemState {
    pub item_id: String,
    pub is_read: bool,
    pub is_starred: bool,
    pub read_at: Option<DateTime<Utc>>,
    pub starred_at: Option<DateTime<Utc>>,
}

impl ItemState {
    pub fn new(item_id: String) -> Self {
        Self {
            item_id,
            is_read: false,
            is_starred: false,
            read_at: None,
            starred_at: None,
        }
    }
}

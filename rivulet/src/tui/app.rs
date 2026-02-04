use crate::domain::{Feed, Item, ItemState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivePane {
    Feeds,
    Items,
    Preview,
}

impl ActivePane {
    pub fn next(self) -> Self {
        match self {
            ActivePane::Feeds => ActivePane::Items,
            ActivePane::Items => ActivePane::Preview,
            ActivePane::Preview => ActivePane::Feeds,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            ActivePane::Feeds => ActivePane::Preview,
            ActivePane::Items => ActivePane::Feeds,
            ActivePane::Preview => ActivePane::Items,
        }
    }
}

pub struct TuiApp {
    pub active_pane: ActivePane,
    pub feeds: Vec<Feed>,
    pub items: Vec<Item>,
    pub item_states: std::collections::HashMap<String, ItemState>,
    pub feed_index: usize,
    pub item_index: usize,
    pub preview_scroll: u16,
    pub should_quit: bool,
    pub status_message: Option<String>,
    pub is_refreshing: bool,
}

impl TuiApp {
    pub fn new() -> Self {
        Self {
            active_pane: ActivePane::Feeds,
            feeds: Vec::new(),
            items: Vec::new(),
            item_states: std::collections::HashMap::new(),
            feed_index: 0,
            item_index: 0,
            preview_scroll: 0,
            should_quit: false,
            status_message: None,
            is_refreshing: false,
        }
    }

    pub fn selected_feed(&self) -> Option<&Feed> {
        self.feeds.get(self.feed_index)
    }

    pub fn selected_item(&self) -> Option<&Item> {
        self.items.get(self.item_index)
    }

    pub fn is_item_read(&self, item_id: &str) -> bool {
        self.item_states
            .get(item_id)
            .map(|s| s.is_read)
            .unwrap_or(false)
    }

    pub fn is_item_starred(&self, item_id: &str) -> bool {
        self.item_states
            .get(item_id)
            .map(|s| s.is_starred)
            .unwrap_or(false)
    }

    pub fn move_up(&mut self) {
        match self.active_pane {
            ActivePane::Feeds => {
                if self.feed_index > 0 {
                    self.feed_index -= 1;
                }
            }
            ActivePane::Items => {
                if self.item_index > 0 {
                    self.item_index -= 1;
                    self.preview_scroll = 0;
                }
            }
            ActivePane::Preview => {
                if self.preview_scroll > 0 {
                    self.preview_scroll = self.preview_scroll.saturating_sub(1);
                }
            }
        }
    }

    pub fn move_down(&mut self) {
        match self.active_pane {
            ActivePane::Feeds => {
                if !self.feeds.is_empty() && self.feed_index < self.feeds.len() - 1 {
                    self.feed_index += 1;
                }
            }
            ActivePane::Items => {
                if !self.items.is_empty() && self.item_index < self.items.len() - 1 {
                    self.item_index += 1;
                    self.preview_scroll = 0;
                }
            }
            ActivePane::Preview => {
                self.preview_scroll = self.preview_scroll.saturating_add(1);
            }
        }
    }

    pub fn set_status(&mut self, message: String) {
        self.status_message = Some(message);
    }

    pub fn clear_status(&mut self) {
        self.status_message = None;
    }
}

impl Default for TuiApp {
    fn default() -> Self {
        Self::new()
    }
}

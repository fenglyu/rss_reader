use crossterm::event::{self, Event, KeyEvent};
use std::time::Duration;

use crate::app::Result;

pub enum AppEvent {
    Key(KeyEvent),
    Tick,
}

pub struct EventHandler {
    tick_rate: Duration,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        Self { tick_rate }
    }

    pub fn next(&self) -> Result<AppEvent> {
        if event::poll(self.tick_rate)? {
            if let Event::Key(key) = event::read()? {
                return Ok(AppEvent::Key(key));
            }
        }
        Ok(AppEvent::Tick)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    MoveUp,
    MoveDown,
    NextPage,
    PrevPage,
    NextPane,
    PrevPane,
    Select,
    ToggleRead,
    ToggleStar,
    OpenInBrowser,
    Refresh,
    ToggleMaximize,
    DeleteFeed,
    None,
}

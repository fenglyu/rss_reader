use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
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
    None,
}

impl From<KeyEvent> for Action {
    fn from(key: KeyEvent) -> Self {
        match key.code {
            KeyCode::Char('q') => Action::Quit,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,
            KeyCode::Char('j') | KeyCode::Down => Action::MoveDown,
            KeyCode::Char('k') | KeyCode::Up => Action::MoveUp,
            KeyCode::Char('n') | KeyCode::PageDown => Action::NextPage,
            KeyCode::Char('p') | KeyCode::PageUp => Action::PrevPage,
            KeyCode::Tab => Action::NextPane,
            KeyCode::BackTab => Action::PrevPane,
            KeyCode::Enter => Action::Select,
            KeyCode::Char('r') => Action::ToggleRead,
            KeyCode::Char('s') => Action::ToggleStar,
            KeyCode::Char('o') => Action::OpenInBrowser,
            KeyCode::Char('R') => Action::Refresh,
            KeyCode::Char('m') => Action::ToggleMaximize,
            _ => Action::None,
        }
    }
}

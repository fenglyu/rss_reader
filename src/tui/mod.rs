pub mod app;
pub mod event;
pub mod layout;

use std::io::{self, Stdout};
use std::sync::Arc;
use std::time::Duration;

use crossterm::{
    event::KeyCode,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::app::{AppContext, Result};
use crate::config::Config;
use crate::scraper::{ChromeScraper, Scraper};
use crate::store::Store;

use self::app::{ActivePane, ItemView, TuiApp};
use self::event::{Action, AppEvent, EventHandler};

type Tui = Terminal<CrosstermBackend<Stdout>>;

pub async fn run(ctx: Arc<AppContext>, config: Arc<Config>) -> Result<()> {
    let mut terminal = setup_terminal()?;
    let result = run_app(&mut terminal, ctx, config).await;
    restore_terminal(&mut terminal)?;
    result
}

fn setup_terminal() -> Result<Tui> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Tui) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

async fn run_app(terminal: &mut Tui, ctx: Arc<AppContext>, config: Arc<Config>) -> Result<()> {
    let mut tui_app = TuiApp::new();
    let mut event_handler = EventHandler::new(Duration::from_millis(100));

    // Load initial data
    load_feeds(&mut tui_app, &ctx)?;
    load_current_items(&mut tui_app, &ctx)?;

    loop {
        terminal.draw(|frame| layout::render(frame, &mut tui_app, &config.colors))?;

        let event = match event_handler.next().await {
            Some(e) => e,
            None => break,
        };

        match event {
            AppEvent::Key(key) => {
                // Handle pending delete confirmation
                if let Some((feed_id, feed_title)) = tui_app.pending_delete.take() {
                    match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                            ctx.store.delete_feed(feed_id)?;
                            load_feeds(&mut tui_app, &ctx)?;
                            load_current_items(&mut tui_app, &ctx)?;
                            tui_app.set_status(format!("Deleted feed: {}", feed_title));
                        }
                        _ => {
                            tui_app.set_status("Delete cancelled".to_string());
                        }
                    }
                    continue;
                }

                let action = config.keybindings.get_action(&key);
                match action {
                    Action::Quit => {
                        tui_app.should_quit = true;
                    }
                    Action::MoveUp => {
                        tui_app.move_up();
                    }
                    Action::MoveDown => {
                        tui_app.move_down();
                    }
                    Action::NextPage => {
                        tui_app.next_page();
                    }
                    Action::PrevPage => {
                        tui_app.prev_page();
                    }
                    Action::ToggleMaximize => {
                        tui_app.toggle_maximize();
                    }
                    Action::NextPane => {
                        tui_app.active_pane = tui_app.active_pane.next();
                    }
                    Action::PrevPane => {
                        tui_app.active_pane = tui_app.active_pane.prev();
                    }
                    Action::Select => {
                        if tui_app.active_pane == ActivePane::Feeds {
                            let feed_id = tui_app.selected_feed().map(|f| f.id);
                            if let Some(feed_id) = feed_id {
                                load_items_for_feed(&mut tui_app, &ctx, feed_id)?;
                                tui_app.active_pane = ActivePane::Items;
                            }
                        }
                    }
                    Action::ToggleRead => {
                        if let Some(item) = tui_app.selected_item() {
                            let item_id = item.id.clone();
                            let is_read = tui_app.is_item_read(&item_id);
                            ctx.store.set_read(&item_id, !is_read)?;
                            update_item_state(&mut tui_app, item_id, |state| {
                                state.is_read = !is_read;
                            });
                        }
                    }
                    Action::ToggleStar => {
                        if let Some(item) = tui_app.selected_item() {
                            let item_id = item.id.clone();
                            let is_starred = tui_app.is_item_starred(&item_id);
                            ctx.store.set_starred(&item_id, !is_starred)?;
                            update_item_state(&mut tui_app, item_id, |state| {
                                state.is_starred = !is_starred;
                            });
                        }
                    }
                    Action::ToggleQueued => {
                        if let Some(item) = tui_app.selected_item() {
                            let item_id = item.id.clone();
                            let is_queued = tui_app.is_item_queued(&item_id);
                            ctx.store.set_queued(&item_id, !is_queued)?;
                            update_item_state(&mut tui_app, item_id, |state| {
                                state.is_queued = !is_queued;
                            });
                        }
                    }
                    Action::ToggleSaved => {
                        if let Some(item) = tui_app.selected_item() {
                            let item_id = item.id.clone();
                            let is_saved = tui_app.is_item_saved(&item_id);
                            ctx.store.set_saved(&item_id, !is_saved)?;
                            update_item_state(&mut tui_app, item_id, |state| {
                                state.is_saved = !is_saved;
                            });
                        }
                    }
                    Action::ToggleArchived => {
                        if let Some(item) = tui_app.selected_item() {
                            let item_id = item.id.clone();
                            let is_archived = tui_app.is_item_archived(&item_id);
                            ctx.store.set_archived(&item_id, !is_archived)?;
                            update_item_state(&mut tui_app, item_id, |state| {
                                state.is_archived = !is_archived;
                            });
                        }
                    }
                    Action::ViewAll => set_item_view(&mut tui_app, &ctx, ItemView::All)?,
                    Action::ViewUnread => set_item_view(&mut tui_app, &ctx, ItemView::Unread)?,
                    Action::ViewStarred => set_item_view(&mut tui_app, &ctx, ItemView::Starred)?,
                    Action::ViewQueued => set_item_view(&mut tui_app, &ctx, ItemView::Queued)?,
                    Action::ViewSaved => set_item_view(&mut tui_app, &ctx, ItemView::Saved)?,
                    Action::ViewArchived => set_item_view(&mut tui_app, &ctx, ItemView::Archived)?,
                    Action::OpenInBrowser => {
                        if let Some(item) = tui_app.selected_item() {
                            if let Some(link) = &item.link {
                                if let Err(e) = open::that(link) {
                                    tui_app.set_status(format!("Failed to open browser: {}", e));
                                } else {
                                    // Mark as read when opened
                                    let item_id = item.id.clone();
                                    ctx.store.set_read(&item_id, true)?;
                                    update_item_state(&mut tui_app, item_id, |state| {
                                        state.is_read = true;
                                    });
                                }
                            }
                        }
                    }
                    Action::Refresh => {
                        if !tui_app.is_refreshing {
                            tui_app.is_refreshing = true;
                            tui_app.refresh_progress = (0, 0);
                            
                            let tx = event_handler.get_tx();
                            let ctx_clone = ctx.clone();
                            
                            let (progress_tx, mut progress_rx) = tokio::sync::mpsc::unbounded_channel::<(usize, usize)>();
                            let tx_clone = tx.clone();
                            tokio::spawn(async move {
                                while let Some((current, total)) = progress_rx.recv().await {
                                    let _ = tx_clone.send(AppEvent::RefreshProgress(current, total));
                                }
                            });
                            
                            tokio::spawn(async move {
                                let feeds = match ctx_clone.store.get_all_feeds() {
                                    Ok(f) => f,
                                    Err(e) => {
                                        tracing::error!("Failed to get feeds: {}", e);
                                        return;
                                    }
                                };
                                
                                let results = ctx_clone
                                    .parallel_fetcher
                                    .fetch_all(feeds, ctx_clone.store.clone(), &ctx_clone.normalizer, Some(progress_tx))
                                    .await;
                                
                                let _ = tx.send(AppEvent::RefreshComplete(results));
                            });
                        }
                    }
                    Action::DeleteFeed => {
                        if tui_app.active_pane == ActivePane::Feeds {
                            if let Some(feed) = tui_app.selected_feed() {
                                let feed_id = feed.id;
                                let feed_title = feed.display_title().to_string();
                                tui_app.pending_delete = Some((feed_id, feed_title));
                            }
                        }
                    }
                    Action::None => {}
                }
            }
            AppEvent::Tick => {
                // Clear status message after some time could be implemented here
            }
            AppEvent::RefreshProgress(current, total) => {
                tui_app.refresh_progress = (current, total);
            }
            AppEvent::RefreshComplete(results) => {
                let mut total_new = 0;
                let mut updated_feed_ids = Vec::new();
                for (feed_id, result) in results {
                    if let Ok(count) = result {
                        total_new += count;
                        if count > 0 {
                            updated_feed_ids.push(feed_id);
                        }
                    }
                }

                // Queue items for background scraping (non-blocking)
                if ctx.scraper_handle.is_some() && !updated_feed_ids.is_empty() {
                    let mut items_to_scrape = Vec::new();
                    for feed_id in updated_feed_ids {
                        if let Ok(items) = ctx.store.get_items_by_feed(feed_id) {
                            items_to_scrape.extend(
                                items.into_iter().filter(ChromeScraper::needs_scraping),
                            );
                        }
                    }
                    if !items_to_scrape.is_empty() {
                        let ctx_clone = ctx.clone();
                        tokio::spawn(async move {
                            ctx_clone.queue_for_scraping(items_to_scrape).await;
                        });
                    }
                }

                load_feeds(&mut tui_app, &ctx)?;
                load_current_items(&mut tui_app, &ctx)?;

                tui_app.is_refreshing = false;
                tui_app.set_status(format!("Refreshed: {} new items", total_new));
            }
        }

        if tui_app.should_quit {
            break;
        }
    }

    Ok(())
}

fn load_feeds(tui_app: &mut TuiApp, ctx: &AppContext) -> Result<()> {
    tui_app.feeds = ctx.store.get_all_feeds()?;
    if tui_app.feed_index >= tui_app.feeds.len() && !tui_app.feeds.is_empty() {
        tui_app.feed_index = tui_app.feeds.len() - 1;
    }
    tui_app.feed_list_state.select(Some(tui_app.feed_index));
    Ok(())
}

fn load_current_items(tui_app: &mut TuiApp, ctx: &AppContext) -> Result<()> {
    tui_app.items = ctx.store.get_items_by_filter(tui_app.item_view.filter())?;
    tui_app.item_states.clear();
    for item in &tui_app.items {
        if let Some(state) = ctx.store.get_item_state(&item.id)? {
            tui_app.item_states.insert(item.id.clone(), state);
        }
    }
    if tui_app.item_index >= tui_app.items.len() && !tui_app.items.is_empty() {
        tui_app.item_index = tui_app.items.len() - 1;
    }
    tui_app.item_list_state.select(Some(tui_app.item_index));
    Ok(())
}

fn set_item_view(tui_app: &mut TuiApp, ctx: &AppContext, view: ItemView) -> Result<()> {
    tui_app.item_view = view;
    tui_app.item_index = 0;
    tui_app.preview_scroll = 0;
    load_current_items(tui_app, ctx)?;
    tui_app.active_pane = ActivePane::Items;
    tui_app.set_status(format!("Showing {} items", view.label()));
    Ok(())
}

fn load_items_for_feed(tui_app: &mut TuiApp, ctx: &AppContext, feed_id: i64) -> Result<()> {
    tui_app.item_view = ItemView::All;
    tui_app.items = ctx.store.get_items_by_feed(feed_id)?;
    tui_app.item_index = 0;
    tui_app.item_states.clear();
    for item in &tui_app.items {
        if let Some(state) = ctx.store.get_item_state(&item.id)? {
            tui_app.item_states.insert(item.id.clone(), state);
        }
    }
    tui_app.item_list_state.select(Some(0));
    Ok(())
}

fn update_item_state(
    tui_app: &mut TuiApp,
    item_id: String,
    update: impl FnOnce(&mut crate::domain::ItemState),
) {
    let state = tui_app
        .item_states
        .entry(item_id.clone())
        .or_insert_with(|| crate::domain::ItemState::new(item_id));
    update(state);
}

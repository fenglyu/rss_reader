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
use crate::store::{RefreshSource, Store};

use self::app::{ActivePane, AppTab, FeedPanelState, ItemView, PendingChord, TuiApp};
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
    tui_app.recent_days = config.ui.latest.days;
    tui_app.recent_limit = config.ui.latest.limit;
    let mut event_handler = EventHandler::new(Duration::from_millis(100));

    // Load initial data
    load_feeds(&mut tui_app, &ctx)?;
    load_reader_items(&mut tui_app, &ctx)?;
    load_latest_items(&mut tui_app, &ctx)?;

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
                            load_reader_items(&mut tui_app, &ctx)?;
                            load_latest_items(&mut tui_app, &ctx)?;
                            tui_app.set_status(format!("Deleted feed: {}", feed_title));
                        }
                        _ => {
                            tui_app.set_status("Delete cancelled".to_string());
                        }
                    }
                    continue;
                }

                // Handle pending multi-key chord (e.g. Ctrl+W <h|l>)
                if let Some(chord) = tui_app.pending_chord.take() {
                    match chord {
                        PendingChord::Window => handle_window_chord_key(&mut tui_app, &key),
                    }
                    continue;
                }

                let action = config.keybindings.get_action(&key);
                let feed_index_before = tui_app.feed_index;
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
                    Action::MoveTop => {
                        tui_app.move_top();
                    }
                    Action::MoveBottom => {
                        tui_app.move_bottom();
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
                        tui_app.active_pane = next_pane_for_tab(&tui_app);
                    }
                    Action::PrevPane => {
                        tui_app.active_pane = prev_pane_for_tab(&tui_app);
                    }
                    Action::Select => {
                        if tui_app.active_tab == AppTab::Reader
                            && tui_app.active_pane == ActivePane::Feeds
                        {
                            let feed_id = tui_app.selected_feed().map(|f| f.id);
                            if let Some(feed_id) = feed_id {
                                tui_app.selected_reader_feed_id = Some(feed_id);
                                load_items_for_feed(&mut tui_app, &ctx, feed_id)?;
                                tui_app.active_pane = ActivePane::Items;
                            }
                        }
                    }
                    Action::ToggleRead => {
                        if let Some(item) = tui_app.selected_item_for_active_tab() {
                            let item_id = item.id.clone();
                            let is_read = tui_app.is_item_read(&item_id);
                            ctx.store.set_read(&item_id, !is_read)?;
                            update_item_state(&mut tui_app, item_id, |state| {
                                state.is_read = !is_read;
                            });
                        }
                    }
                    Action::ToggleStar => {
                        if let Some(item) = tui_app.selected_item_for_active_tab() {
                            let item_id = item.id.clone();
                            let is_starred = tui_app.is_item_starred(&item_id);
                            ctx.store.set_starred(&item_id, !is_starred)?;
                            update_item_state(&mut tui_app, item_id, |state| {
                                state.is_starred = !is_starred;
                            });
                        }
                    }
                    Action::ToggleQueued => {
                        if let Some(item) = tui_app.selected_item_for_active_tab() {
                            let item_id = item.id.clone();
                            let is_queued = tui_app.is_item_queued(&item_id);
                            ctx.store.set_queued(&item_id, !is_queued)?;
                            update_item_state(&mut tui_app, item_id, |state| {
                                state.is_queued = !is_queued;
                            });
                        }
                    }
                    Action::ToggleSaved => {
                        if let Some(item) = tui_app.selected_item_for_active_tab() {
                            let item_id = item.id.clone();
                            let is_saved = tui_app.is_item_saved(&item_id);
                            ctx.store.set_saved(&item_id, !is_saved)?;
                            update_item_state(&mut tui_app, item_id, |state| {
                                state.is_saved = !is_saved;
                            });
                        }
                    }
                    Action::ToggleArchived => {
                        if let Some(item) = tui_app.selected_item_for_active_tab() {
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
                    Action::ViewLatest => {
                        tui_app.active_tab = AppTab::Latest;
                        tui_app.active_pane = ActivePane::Items;
                        load_latest_items(&mut tui_app, &ctx)?;
                    }
                    Action::ViewReader => {
                        tui_app.active_tab = AppTab::Reader;
                        tui_app.feed_panel = FeedPanelState::Expanded;
                        tui_app.active_pane = ActivePane::Feeds;
                        load_reader_items(&mut tui_app, &ctx)?;
                    }
                    Action::OpenInBrowser => {
                        if let Some(item) = tui_app.selected_item_for_active_tab() {
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

                            let (progress_tx, mut progress_rx) =
                                tokio::sync::mpsc::unbounded_channel::<(usize, usize)>();
                            let tx_clone = tx.clone();
                            tokio::spawn(async move {
                                while let Some((current, total)) = progress_rx.recv().await {
                                    let _ =
                                        tx_clone.send(AppEvent::RefreshProgress(current, total));
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
                                let run_id = match ctx_clone
                                    .store
                                    .begin_refresh_run(RefreshSource::Tui, feeds.len())
                                {
                                    Ok(run_id) => run_id,
                                    Err(e) => {
                                        tracing::error!("Failed to start refresh run: {}", e);
                                        return;
                                    }
                                };

                                let results = ctx_clone
                                    .parallel_fetcher
                                    .fetch_all(
                                        feeds,
                                        ctx_clone.store.clone(),
                                        &ctx_clone.normalizer,
                                        Some(progress_tx),
                                    )
                                    .await;

                                let _ = tx.send(AppEvent::RefreshComplete(run_id, results));
                            });
                        }
                    }
                    Action::ToggleFeedPanel => {
                        if tui_app.active_tab == AppTab::Reader {
                            tui_app.feed_panel = match tui_app.feed_panel {
                                FeedPanelState::Collapsed => {
                                    tui_app.active_pane = ActivePane::Feeds;
                                    FeedPanelState::Expanded
                                }
                                FeedPanelState::Expanded => {
                                    tui_app.active_pane =
                                        if tui_app.selected_reader_feed_id.is_some() {
                                            ActivePane::Items
                                        } else {
                                            ActivePane::Preview
                                        };
                                    FeedPanelState::Collapsed
                                }
                            };
                        } else {
                            tui_app.set_status(
                                "Feed panel is Reader-only - press Alt+2 to switch".to_string(),
                            );
                        }
                    }
                    Action::DeleteFeed => {
                        if tui_app.active_tab == AppTab::Reader
                            && tui_app.active_pane == ActivePane::Feeds
                        {
                            if let Some(feed) = tui_app.selected_feed() {
                                let feed_id = feed.id;
                                let feed_title = feed.display_title().to_string();
                                tui_app.pending_delete = Some((feed_id, feed_title));
                            }
                        }
                    }
                    Action::WindowChord => {
                        tui_app.pending_chord = Some(PendingChord::Window);
                        tui_app.set_status(
                            "-- WINDOW -- (h: left, l: right, Esc: cancel)".to_string(),
                        );
                    }
                    Action::None => {}
                }

                // If feed-pane navigation moved the highlight to a new feed,
                // auto-load that feed's items so the Items pane (and its filter
                // count in the title) reflects the highlighted feed without
                // requiring an explicit Enter.
                if tui_app.active_tab == AppTab::Reader
                    && tui_app.active_pane == ActivePane::Feeds
                    && tui_app.feed_index != feed_index_before
                {
                    if let Some(feed_id) = tui_app.selected_feed().map(|f| f.id) {
                        if tui_app.selected_reader_feed_id != Some(feed_id) {
                            tui_app.selected_reader_feed_id = Some(feed_id);
                            load_items_for_feed(&mut tui_app, &ctx, feed_id)?;
                        }
                    }
                }
            }
            AppEvent::Tick => {
                // Clear status message after some time could be implemented here
            }
            AppEvent::RefreshProgress(current, total) => {
                tui_app.refresh_progress = (current, total);
            }
            AppEvent::RefreshComplete(run_id, results) => {
                let mut total_new = 0;
                let mut errors = 0;
                let mut updated_feed_ids = Vec::new();
                for (feed_id, result) in results {
                    if let Ok(refresh) = result {
                        total_new += refresh.new_count;
                        ctx.store.record_refresh_run_items(
                            run_id,
                            feed_id,
                            &refresh.inserted_item_ids,
                        )?;
                        if refresh.new_count > 0 {
                            updated_feed_ids.push(feed_id);
                        }
                    } else {
                        errors += 1;
                    }
                }
                ctx.store.complete_refresh_run(run_id, total_new, errors)?;

                // Queue items for background scraping (non-blocking)
                if ctx.scraper_handle.is_some() && !updated_feed_ids.is_empty() {
                    let mut items_to_scrape = Vec::new();
                    for feed_id in updated_feed_ids {
                        if let Ok(items) = ctx.store.get_items_by_feed(feed_id) {
                            items_to_scrape
                                .extend(items.into_iter().filter(ChromeScraper::needs_scraping));
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
                load_reader_items(&mut tui_app, &ctx)?;
                load_latest_items(&mut tui_app, &ctx)?;

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
    if let Some(feed_id) = tui_app.selected_reader_feed_id {
        if !tui_app.feeds.iter().any(|feed| feed.id == feed_id) {
            tui_app.selected_reader_feed_id = None;
            tui_app.items.clear();
            tui_app.item_index = 0;
            tui_app.item_list_state.select(None);
        }
    }
    if tui_app.feed_index >= tui_app.feeds.len() && !tui_app.feeds.is_empty() {
        tui_app.feed_index = tui_app.feeds.len() - 1;
    }
    tui_app.feed_list_state.select(Some(tui_app.feed_index));
    Ok(())
}

fn load_reader_items(tui_app: &mut TuiApp, ctx: &AppContext) -> Result<()> {
    if let Some(feed_id) = tui_app.selected_reader_feed_id {
        load_items_for_feed(tui_app, ctx, feed_id)?;
    } else {
        tui_app.items.clear();
        tui_app.item_index = 0;
        tui_app.item_list_state.select(None);
        reload_item_states(tui_app, ctx)?;
    }
    Ok(())
}

fn finish_reader_items_load(tui_app: &mut TuiApp, ctx: &AppContext) -> Result<()> {
    if tui_app.item_index >= tui_app.items.len() && !tui_app.items.is_empty() {
        tui_app.item_index = tui_app.items.len() - 1;
    }
    if tui_app.items.is_empty() {
        tui_app.item_list_state.select(None);
    } else {
        tui_app.item_list_state.select(Some(tui_app.item_index));
    }
    reload_item_states(tui_app, ctx)?;
    Ok(())
}

fn load_latest_items(tui_app: &mut TuiApp, ctx: &AppContext) -> Result<()> {
    tui_app.latest_run_id = ctx.store.get_latest_refresh_run_id()?;
    tui_app.latest_items = ctx.store.get_recent_items(
        tui_app.item_view.filter(),
        tui_app.recent_days,
        tui_app.recent_limit,
        tui_app.latest_run_id,
    )?;
    if tui_app.latest_index >= tui_app.latest_items.len() && !tui_app.latest_items.is_empty() {
        tui_app.latest_index = tui_app.latest_items.len() - 1;
    }
    tui_app.latest_list_state.select(Some(tui_app.latest_index));
    reload_item_states(tui_app, ctx)?;
    Ok(())
}

fn set_item_view(tui_app: &mut TuiApp, ctx: &AppContext, view: ItemView) -> Result<()> {
    tui_app.item_view = view;
    tui_app.item_index = 0;
    tui_app.latest_index = 0;
    tui_app.preview_scroll = 0;
    match tui_app.active_tab {
        AppTab::Latest => load_latest_items(tui_app, ctx)?,
        AppTab::Reader => load_reader_items(tui_app, ctx)?,
    }
    tui_app.active_pane =
        if tui_app.active_tab == AppTab::Reader && tui_app.selected_reader_feed_id.is_none() {
            ActivePane::Feeds
        } else {
            ActivePane::Items
        };
    tui_app.set_status(format!("Showing {} items", view.label()));
    Ok(())
}

fn load_items_for_feed(tui_app: &mut TuiApp, ctx: &AppContext, feed_id: i64) -> Result<()> {
    let items = ctx.store.get_items_by_feed(feed_id)?;
    let filter = tui_app.item_view.filter();
    let mut filtered = Vec::new();
    for item in items {
        if item_matches_filter(ctx, &item, filter)? {
            filtered.push(item);
        }
    }
    tui_app.items = filtered;
    tui_app.item_index = 0;
    finish_reader_items_load(tui_app, ctx)
}

fn item_matches_filter(
    ctx: &AppContext,
    item: &crate::domain::Item,
    filter: crate::store::ItemListFilter,
) -> Result<bool> {
    let state = ctx.store.get_item_state(&item.id)?;
    let is_read = state.as_ref().map(|s| s.is_read).unwrap_or(false);
    let is_starred = state.as_ref().map(|s| s.is_starred).unwrap_or(false);
    let is_queued = state.as_ref().map(|s| s.is_queued).unwrap_or(false);
    let is_saved = state.as_ref().map(|s| s.is_saved).unwrap_or(false);
    let is_archived = state.as_ref().map(|s| s.is_archived).unwrap_or(false);

    Ok(match filter {
        crate::store::ItemListFilter::All => !is_archived,
        crate::store::ItemListFilter::Unread => !is_archived && !is_read,
        crate::store::ItemListFilter::Starred => !is_archived && is_starred,
        crate::store::ItemListFilter::Queued => !is_archived && is_queued,
        crate::store::ItemListFilter::Saved => !is_archived && is_saved,
        crate::store::ItemListFilter::Archived => is_archived,
    })
}

fn reload_item_states(tui_app: &mut TuiApp, ctx: &AppContext) -> Result<()> {
    let mut ids: Vec<String> = tui_app.items.iter().map(|item| item.id.clone()).collect();
    ids.extend(
        tui_app
            .latest_items
            .iter()
            .map(|recent| recent.item.id.clone()),
    );
    ids.sort();
    ids.dedup();

    tui_app.item_states.clear();
    for item_id in ids {
        if let Some(state) = ctx.store.get_item_state(&item_id)? {
            tui_app.item_states.insert(item_id, state);
        }
    }
    Ok(())
}

fn next_pane_for_tab(tui_app: &TuiApp) -> ActivePane {
    match tui_app.active_tab {
        AppTab::Latest => match tui_app.active_pane {
            ActivePane::Items => ActivePane::Preview,
            _ => ActivePane::Items,
        },
        AppTab::Reader => match tui_app.feed_panel {
            FeedPanelState::Expanded => tui_app.active_pane.next(),
            FeedPanelState::Collapsed => match tui_app.active_pane {
                ActivePane::Items => ActivePane::Preview,
                _ => ActivePane::Items,
            },
        },
    }
}

fn prev_pane_for_tab(tui_app: &TuiApp) -> ActivePane {
    match tui_app.active_tab {
        AppTab::Latest => match tui_app.active_pane {
            ActivePane::Preview => ActivePane::Items,
            _ => ActivePane::Preview,
        },
        AppTab::Reader => match tui_app.feed_panel {
            FeedPanelState::Expanded => tui_app.active_pane.prev(),
            FeedPanelState::Collapsed => match tui_app.active_pane {
                ActivePane::Preview => ActivePane::Items,
                _ => ActivePane::Preview,
            },
        },
    }
}

fn handle_window_chord_key(tui_app: &mut TuiApp, key: &crossterm::event::KeyEvent) {
    if tui_app.maximized {
        tui_app.set_status("Window chord unavailable in maximize mode".to_string());
        return;
    }
    match key.code {
        KeyCode::Char('h') | KeyCode::Left => {
            let target = focus_left_for_tab(tui_app);
            if target == tui_app.active_pane {
                tui_app.set_status("Already at leftmost pane".to_string());
            } else {
                tui_app.active_pane = target;
                tui_app.clear_status();
            }
        }
        KeyCode::Char('l') | KeyCode::Right => {
            let target = focus_right_for_tab(tui_app);
            if target == tui_app.active_pane {
                tui_app.set_status("Already at rightmost pane".to_string());
            } else {
                tui_app.active_pane = target;
                tui_app.clear_status();
            }
        }
        KeyCode::Esc => {
            tui_app.set_status("Window chord cancelled".to_string());
        }
        _ => {
            tui_app.clear_status();
        }
    }
}

fn focus_left_for_tab(tui_app: &TuiApp) -> ActivePane {
    match tui_app.active_tab {
        AppTab::Latest => match tui_app.active_pane {
            ActivePane::Preview => ActivePane::Items,
            other => other,
        },
        AppTab::Reader => {
            let feeds_focusable = tui_app.feed_panel == FeedPanelState::Expanded;
            let items_focusable = tui_app.selected_reader_feed_id.is_some();
            match tui_app.active_pane {
                ActivePane::Preview => {
                    if items_focusable {
                        ActivePane::Items
                    } else if feeds_focusable {
                        ActivePane::Feeds
                    } else {
                        ActivePane::Preview
                    }
                }
                ActivePane::Items => {
                    if feeds_focusable {
                        ActivePane::Feeds
                    } else {
                        ActivePane::Items
                    }
                }
                ActivePane::Feeds => ActivePane::Feeds,
            }
        }
    }
}

fn focus_right_for_tab(tui_app: &TuiApp) -> ActivePane {
    match tui_app.active_tab {
        AppTab::Latest => match tui_app.active_pane {
            ActivePane::Items => ActivePane::Preview,
            other => other,
        },
        AppTab::Reader => {
            let items_focusable = tui_app.selected_reader_feed_id.is_some();
            match tui_app.active_pane {
                ActivePane::Feeds => {
                    if items_focusable {
                        ActivePane::Items
                    } else {
                        ActivePane::Preview
                    }
                }
                ActivePane::Items => ActivePane::Preview,
                ActivePane::Preview => ActivePane::Preview,
            }
        }
    }
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

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::config::ColorConfig;
use crate::tui::app::{ActivePane, TuiApp};

pub fn render(frame: &mut Frame, app: &mut TuiApp, colors: &ColorConfig) {
    if app.maximized {
        // Maximized mode: only preview and status bar
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10),   // Preview pane (full height)
                Constraint::Length(1), // Status bar
            ])
            .split(frame.area());

        render_preview_pane(frame, app, chunks[0], colors);
        render_status_bar(frame, app, chunks[1], colors);
    } else {
        // Normal mode: all panes
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(10),     // Feeds pane
                Constraint::Percentage(40), // Items pane
                Constraint::Min(10),        // Preview pane
                Constraint::Length(1),      // Status bar
            ])
            .split(frame.area());

        render_feeds_pane(frame, app, chunks[0], colors);
        render_items_pane(frame, app, chunks[1], colors);
        render_preview_pane(frame, app, chunks[2], colors);
        render_status_bar(frame, app, chunks[3], colors);
    }
}

fn render_feeds_pane(frame: &mut Frame, app: &mut TuiApp, area: Rect, colors: &ColorConfig) {
    let is_active = app.active_pane == ActivePane::Feeds;
    let border_style = if is_active {
        Style::default().fg(colors.active_border)
    } else {
        Style::default().fg(colors.inactive_border)
    };

    let items: Vec<ListItem> = app
        .feeds
        .iter()
        .map(|feed| {
            let unread = app
                .items
                .iter()
                .filter(|item| item.feed_id == feed.id && !app.is_item_read(&item.id))
                .count();

            let content = if unread > 0 {
                format!("{} ({})", feed.display_title(), unread)
            } else {
                feed.display_title().to_string()
            };

            ListItem::new(content)
        })
        .collect();

    let title = format!(
        " Feeds ({}) [{}/{}] ",
        app.feeds.len(),
        app.feed_index + 1,
        app.feeds.len().max(1)
    );

    let highlight_style = if is_active {
        Style::default()
            .bg(colors.selection_bg_active)
            .fg(colors.selection_fg_active)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .bg(colors.selection_bg_inactive)
            .fg(colors.selection_fg_inactive)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    let list = List::new(items)
        .block(block)
        .highlight_style(highlight_style)
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area, &mut app.feed_list_state);
}

fn render_items_pane(frame: &mut Frame, app: &mut TuiApp, area: Rect, colors: &ColorConfig) {
    let is_active = app.active_pane == ActivePane::Items;
    let border_style = if is_active {
        Style::default().fg(colors.active_border)
    } else {
        Style::default().fg(colors.inactive_border)
    };

    let items: Vec<ListItem> = app
        .items
        .iter()
        .map(|item| {
            let is_read = app.is_item_read(&item.id);
            let is_starred = app.is_item_starred(&item.id);

            let marker = if is_starred {
                "★"
            } else if !is_read {
                "●"
            } else {
                " "
            };

            let date = item
                .published_at
                .map(|d| d.format("%m/%d").to_string())
                .unwrap_or_else(|| "     ".to_string());

            let content = format!("{} {} {}", marker, date, item.display_title());

            let style = if !is_read {
                Style::default()
                    .fg(colors.unread_item)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(colors.read_item)
            };

            ListItem::new(content).style(style)
        })
        .collect();

    let title = format!(
        " Items ({}) [{}/{}] ",
        app.items.len(),
        app.item_index + 1,
        app.items.len().max(1)
    );

    let highlight_style = if is_active {
        Style::default()
            .bg(colors.selection_bg_active)
            .fg(colors.selection_fg_active)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .bg(colors.selection_bg_inactive)
            .fg(colors.selection_fg_inactive)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    let list = List::new(items)
        .block(block)
        .highlight_style(highlight_style)
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area, &mut app.item_list_state);
}

fn render_preview_pane(frame: &mut Frame, app: &TuiApp, area: Rect, colors: &ColorConfig) {
    let is_active = app.active_pane == ActivePane::Preview;
    let border_style = if is_active {
        Style::default().fg(colors.active_border)
    } else {
        Style::default().fg(colors.inactive_border)
    };

    let (title, content) = if let Some(item) = app.selected_item() {
        let title_text = item.display_title().to_string();
        let mut lines = Vec::new();

        // Title
        lines.push(Line::from(Span::styled(
            item.display_title(),
            Style::default().add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        // Metadata
        if let Some(author) = &item.author {
            lines.push(Line::from(Span::styled(
                format!("By: {}", author),
                Style::default().fg(colors.metadata_author),
            )));
        }
        if let Some(date) = item.published_at {
            lines.push(Line::from(Span::styled(
                format!("Date: {}", date.format("%Y-%m-%d %H:%M")),
                Style::default().fg(colors.metadata_date),
            )));
        }
        if let Some(link) = &item.link {
            lines.push(Line::from(Span::styled(
                format!("Link: {}", link),
                Style::default().fg(colors.metadata_link),
            )));
        }
        lines.push(Line::from(""));
        lines.push(Line::from(
            "─".repeat(area.width.saturating_sub(2) as usize),
        ));
        lines.push(Line::from(""));

        // Content
        let content_text = strip_html(item.display_content());
        for line in content_text.lines() {
            lines.push(Line::from(line.to_string()));
        }

        (format!(" {} ", title_text), Text::from(lines))
    } else {
        (" Preview ".to_string(), Text::from("No item selected"))
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    let paragraph = Paragraph::new(content)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((app.preview_scroll, 0));

    frame.render_widget(paragraph, area);
}

fn render_status_bar(frame: &mut Frame, app: &TuiApp, area: Rect, colors: &ColorConfig) {
    let status = if let Some((_, ref title)) = app.pending_delete {
        format!("Delete \"{}\"? (y/n)", title)
    } else if app.is_refreshing {
        "Refreshing feeds...".to_string()
    } else if let Some(ref msg) = app.status_message {
        msg.clone()
    } else if app.maximized {
        "j/k:Scroll  n/p:Page  m:Exit maximize  q:Quit".to_string()
    } else {
        "j/k:Nav  n/p:Page  Tab:Pane  r:Read  s:Star  o:Open  R:Refresh  d:Delete  m:Max  q:Quit"
            .to_string()
    };

    let paragraph =
        Paragraph::new(status).style(Style::default().fg(colors.status_fg).bg(colors.status_bg));

    frame.render_widget(paragraph, area);
}

fn strip_html(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    let mut last_was_space = false;

    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => {
                if c.is_whitespace() {
                    if !last_was_space {
                        result.push(' ');
                        last_was_space = true;
                    }
                } else {
                    result.push(c);
                    last_was_space = false;
                }
            }
            _ => {}
        }
    }

    result.trim().to_string()
}

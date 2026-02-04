use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::tui::app::{ActivePane, TuiApp};

pub fn render(frame: &mut Frame, app: &TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),  // Feeds pane
            Constraint::Percentage(40), // Items pane
            Constraint::Min(10),    // Preview pane
            Constraint::Length(1),  // Status bar
        ])
        .split(frame.area());

    render_feeds_pane(frame, app, chunks[0]);
    render_items_pane(frame, app, chunks[1]);
    render_preview_pane(frame, app, chunks[2]);
    render_status_bar(frame, app, chunks[3]);
}

fn render_feeds_pane(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let is_active = app.active_pane == ActivePane::Feeds;
    let border_style = if is_active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let items: Vec<ListItem> = app
        .feeds
        .iter()
        .enumerate()
        .map(|(i, feed)| {
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

            let style = if i == app.feed_index && is_active {
                Style::default()
                    .bg(Color::Cyan)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else if i == app.feed_index {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            ListItem::new(content).style(style)
        })
        .collect();

    let title = format!(" Feeds ({}) ", app.feeds.len());
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn render_items_pane(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let is_active = app.active_pane == ActivePane::Items;
    let border_style = if is_active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let items: Vec<ListItem> = app
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| {
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

            let base_style = if !is_read {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let style = if i == app.item_index && is_active {
                Style::default()
                    .bg(Color::Cyan)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else if i == app.item_index {
                base_style.bg(Color::DarkGray)
            } else {
                base_style
            };

            ListItem::new(content).style(style)
        })
        .collect();

    let title = format!(" Items ({}) ", app.items.len());
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn render_preview_pane(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let is_active = app.active_pane == ActivePane::Preview;
    let border_style = if is_active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
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
                Style::default().fg(Color::Yellow),
            )));
        }
        if let Some(date) = item.published_at {
            lines.push(Line::from(Span::styled(
                format!("Date: {}", date.format("%Y-%m-%d %H:%M")),
                Style::default().fg(Color::Yellow),
            )));
        }
        if let Some(link) = &item.link {
            lines.push(Line::from(Span::styled(
                format!("Link: {}", link),
                Style::default().fg(Color::Blue),
            )));
        }
        lines.push(Line::from(""));
        lines.push(Line::from("─".repeat(area.width.saturating_sub(2) as usize)));
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

fn render_status_bar(frame: &mut Frame, app: &TuiApp, area: Rect) {
    let status = if app.is_refreshing {
        "Refreshing feeds...".to_string()
    } else if let Some(ref msg) = app.status_message {
        msg.clone()
    } else {
        "j/k:Navigate  Tab:Pane  r:Read  s:Star  o:Open  R:Refresh  q:Quit".to_string()
    };

    let paragraph = Paragraph::new(status).style(Style::default().fg(Color::White).bg(Color::DarkGray));

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

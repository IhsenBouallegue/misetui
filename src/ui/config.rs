use crate::app::{App, LoadState};
use crate::theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let chunks = if app.search_active {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(3)])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(0), Constraint::Min(3)])
            .split(area)
    };

    // Search bar
    if app.search_active {
        let search_block = Block::default()
            .title(Span::styled(" Search ", theme::title()))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(theme::border_focused())
            .style(ratatui::style::Style::default().bg(theme::SURFACE));

        let search = Paragraph::new(Line::from(vec![
            Span::styled("/", theme::key_hint()),
            Span::styled(&app.search_query, theme::search_input()),
            Span::styled("█", theme::search_input()),
        ]))
        .block(search_block);

        f.render_widget(search, chunks[0]);
    }

    let content_area = chunks[1];

    let count = app.filtered_configs.len();
    let total = app.configs.len();
    let title = if app.search_active && !app.search_query.is_empty() {
        format!(" Config ({count}/{total}) ")
    } else {
        format!(" Config ({total}) ")
    };

    let block = Block::default()
        .title(Span::styled(title, theme::title()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::border_focused())
        .style(ratatui::style::Style::default().bg(theme::BG));

    if app.config_state == LoadState::Loading {
        let spinner = app.spinner_char();
        let loading = Paragraph::new(format!("  {spinner} Loading config..."))
            .style(theme::muted())
            .block(block);
        f.render_widget(loading, content_area);
        return;
    }

    let configs = app.visible_configs();
    if configs.is_empty() {
        let msg = if app.search_active && !app.search_query.is_empty() {
            "  No matching configs"
        } else {
            "  No config files found"
        };
        let empty = Paragraph::new(msg).style(theme::muted()).block(block);
        f.render_widget(empty, content_area);
        return;
    }

    let items: Vec<ListItem> = configs
        .iter()
        .map(|cfg| {
            let tools_str = if cfg.tools.is_empty() {
                String::new()
            } else {
                format!("  [{}]", cfg.tools.join(", "))
            };
            ListItem::new(Line::from(vec![
                Span::styled("  ", theme::active_indicator()),
                Span::styled(&cfg.path, theme::table_row()),
                Span::styled(tools_str, theme::muted()),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(theme::table_selected());

    let mut state = ListState::default();
    state.select(Some(app.config_selected));
    f.render_stateful_widget(list, content_area, &mut state);
}

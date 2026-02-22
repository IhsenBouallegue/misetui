use crate::app::{App, LoadState};
use crate::theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table, TableState};
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

    let count = app.filtered_registry.len();
    let total = app.registry.len();
    let title = if app.search_active && !app.search_query.is_empty() {
        format!(" Registry ({count}/{total}) ")
    } else {
        format!(" Registry ({total}) ")
    };

    let block = Block::default()
        .title(Span::styled(title, theme::title()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::border_focused())
        .style(ratatui::style::Style::default().bg(theme::BG));

    if app.registry_state == LoadState::Loading {
        let spinner = app.spinner_char();
        let loading = Paragraph::new(format!("  {spinner} Loading registry..."))
            .style(theme::muted())
            .block(block);
        f.render_widget(loading, content_area);
        return;
    }

    let entries = app.visible_registry_entries();
    if entries.is_empty() {
        let empty = Paragraph::new("  No matching tools")
            .style(theme::muted())
            .block(block);
        f.render_widget(empty, content_area);
        return;
    }

    let header = Row::new(vec![
        Cell::from("Name"),
        Cell::from("Description"),
    ])
    .style(theme::table_header());

    let rows: Vec<Row> = entries
        .iter()
        .map(|entry| {
            let desc = entry
                .description
                .as_deref()
                .unwrap_or("")
                .chars()
                .take(60)
                .collect::<String>();
            Row::new(vec![
                Cell::from(Span::styled(&entry.short[..], theme::table_row())),
                Cell::from(Span::styled(desc, theme::muted())),
            ])
        })
        .collect();

    let widths = [Constraint::Length(24), Constraint::Min(20)];

    let table = Table::new(rows, widths)
        .header(header)
        .block(block)
        .row_highlight_style(theme::table_selected());

    let mut state = TableState::default();
    state.select(Some(app.registry_selected));
    f.render_stateful_widget(table, content_area, &mut state);
}

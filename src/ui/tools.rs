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

    let count = app.filtered_tools.len();
    let total = app.tools.len();
    let title = if app.search_active && !app.search_query.is_empty() {
        format!(" Tools ({count}/{total}) ")
    } else {
        format!(" Tools ({total}) ")
    };

    let block = Block::default()
        .title(Span::styled(title, theme::title()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::border_focused())
        .style(ratatui::style::Style::default().bg(theme::BG));

    if app.tools_state == LoadState::Loading {
        let spinner = app.spinner_char();
        let loading = Paragraph::new(format!("  {spinner} Loading tools..."))
            .style(theme::muted())
            .block(block);
        f.render_widget(loading, content_area);
        return;
    }

    let tools = app.visible_tools();
    if tools.is_empty() {
        let msg = if app.search_active && !app.search_query.is_empty() {
            "  No matching tools"
        } else {
            "  No tools installed"
        };
        let empty = Paragraph::new(msg).style(theme::muted()).block(block);
        f.render_widget(empty, content_area);
        return;
    }

    let header = Row::new(vec![
        Cell::from("Name"),
        Cell::from("Version"),
        Cell::from("Status"),
        Cell::from("Source"),
    ])
    .style(theme::table_header());

    let rows: Vec<Row> = tools
        .iter()
        .map(|tool| {
            let status = if tool.active {
                Cell::from(Span::styled("● active", theme::active_indicator()))
            } else {
                Cell::from(Span::styled("○ inactive", theme::inactive_indicator()))
            };

            Row::new(vec![
                Cell::from(Span::styled(&tool.name[..], theme::table_row())),
                Cell::from(Span::styled(&tool.version[..], theme::table_row())),
                status,
                Cell::from(Span::styled(&tool.source[..], theme::muted())),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(16),
        Constraint::Length(14),
        Constraint::Length(12),
        Constraint::Min(10),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(block)
        .row_highlight_style(theme::table_selected());

    let mut state = TableState::default();
    state.select(Some(app.tools_selected));
    f.render_stateful_widget(table, content_area, &mut state);
}

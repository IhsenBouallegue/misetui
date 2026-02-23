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

    let count = app.filtered_outdated.len();
    let total = app.outdated.len();
    let title = if app.search_active && !app.search_query.is_empty() {
        format!(" Outdated ({count}/{total}) ")
    } else {
        format!(" Outdated ({total}) ")
    };

    let block = Block::default()
        .title(Span::styled(title, theme::title()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::border_focused())
        .style(ratatui::style::Style::default().bg(theme::BG));

    if app.outdated_state == LoadState::Loading {
        let spinner = app.spinner_char();
        let loading = Paragraph::new(format!("  {spinner} Loading outdated tools..."))
            .style(theme::muted())
            .block(block);
        f.render_widget(loading, content_area);
        return;
    }

    let outdated = app.visible_outdated();
    if outdated.is_empty() {
        let msg = if app.search_active && !app.search_query.is_empty() {
            "  No matching outdated tools"
        } else {
            "  All tools are up to date!"
        };
        let empty = Paragraph::new(msg).style(theme::muted()).block(block);
        f.render_widget(empty, content_area);
        return;
    }

    let header = Row::new(vec![
        Cell::from(format!("Name{}", app.sort_indicator(0))),
        Cell::from(format!("Current{}", app.sort_indicator(1))),
        Cell::from(format!("Latest{}", app.sort_indicator(2))),
        Cell::from(format!("Requested{}", app.sort_indicator(3))),
    ])
    .style(theme::table_header());

    let rows: Vec<Row> = outdated
        .iter()
        .map(|tool| {
            Row::new(vec![
                Cell::from(Span::styled(&tool.name[..], theme::table_row())),
                Cell::from(Span::styled(&tool.current[..], theme::table_row())),
                Cell::from(Span::styled(
                    &tool.latest[..],
                    theme::active_indicator(),
                )),
                Cell::from(Span::styled(&tool.requested[..], theme::muted())),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(20),
        Constraint::Length(16),
        Constraint::Length(16),
        Constraint::Min(10),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(block)
        .row_highlight_style(theme::table_selected());

    let mut state = TableState::default();
    state.select(Some(app.outdated_selected));
    f.render_stateful_widget(table, content_area, &mut state);
}

use super::highlight::highlight_cached;
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

    let count = app.filtered_tasks.len();
    let total = app.tasks.len();
    let title = if app.search_active && !app.search_query.is_empty() {
        format!(" Tasks ({count}/{total}) ")
    } else {
        format!(" Tasks ({total}) ")
    };

    let block = Block::default()
        .title(Span::styled(title, theme::title()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::border_focused())
        .style(ratatui::style::Style::default().bg(theme::BG));

    if app.tasks_state == LoadState::Loading {
        let spinner = app.spinner_char();
        let loading = Paragraph::new(format!("  {spinner} Loading tasks..."))
            .style(theme::muted())
            .block(block);
        f.render_widget(loading, content_area);
        return;
    }

    let tasks = app.visible_tasks();
    if tasks.is_empty() {
        let msg = if app.search_active && !app.search_query.is_empty() {
            "  No matching tasks"
        } else {
            "  No tasks defined"
        };
        let empty = Paragraph::new(msg).style(theme::muted()).block(block);
        f.render_widget(empty, content_area);
        return;
    }

    let header = Row::new(vec![
        Cell::from(format!("Name{}", app.sort_indicator(0))),
        Cell::from(format!("Description{}", app.sort_indicator(1))),
        Cell::from(format!("Source{}", app.sort_indicator(2))),
    ])
    .style(theme::table_header());

    let rows: Vec<Row> = tasks
        .iter()
        .enumerate()
        .map(|(i, task)| {
            let source_short = task.source.rsplit('/').next().unwrap_or(&task.source);
            let name_hl = app.tasks_hl.get(i).map(|v| v.as_slice()).unwrap_or(&[]);
            Row::new(vec![
                Cell::from(highlight_cached(&task.name, name_hl, theme::table_row())),
                Cell::from(Span::styled(task.description.clone(), theme::table_row())),
                Cell::from(Span::styled(source_short, theme::muted())),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(20),
        Constraint::Min(20),
        Constraint::Length(20),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(block)
        .row_highlight_style(theme::table_selected());

    let mut state = TableState::default();
    state.select(Some(app.tasks_selected));
    f.render_stateful_widget(table, content_area, &mut state);
}

use super::highlight::highlight_cached;
use crate::app::{App, LoadState};
use crate::theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
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
            .style(Style::default().bg(theme::SURFACE));

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
        .style(Style::default().bg(theme::BG));

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
        Cell::from(format!("Name{}", app.sort_indicator(0))),
        Cell::from(format!("Version{}", app.sort_indicator(1))),
        Cell::from(format!("Status{}", app.sort_indicator(2))),
        Cell::from(format!("Source{}", app.sort_indicator(3))),
    ])
    .style(theme::table_header());

    let rows: Vec<Row> = tools
        .iter()
        .enumerate()
        .map(|(i, tool)| {
            let status = if tool.active {
                Cell::from(Span::styled("● active", theme::active_indicator()))
            } else {
                Cell::from(Span::styled("○ inactive", theme::inactive_indicator()))
            };

            // Version cell: show outdated arrow if applicable
            let version_cell =
                if let Some(outdated) = app.outdated_map.get(&tool.name) {
                    if outdated.current == tool.version && outdated.latest != tool.version {
                        Cell::from(Line::from(vec![
                            Span::styled(tool.version.clone(), theme::table_row()),
                            Span::styled(
                                format!(" → {}", outdated.latest),
                                Style::default().fg(theme::YELLOW),
                            ),
                        ]))
                    } else {
                        Cell::from(Span::styled(tool.version.clone(), theme::table_row()))
                    }
                } else {
                    Cell::from(Span::styled(tool.version.clone(), theme::table_row()))
                };

            let name_hl = app.tools_hl.get(i).map(|v| v.as_slice()).unwrap_or(&[]);
            Row::new(vec![
                Cell::from(highlight_cached(&tool.name, name_hl, theme::table_row())),
                version_cell,
                status,
                Cell::from(Span::styled(&tool.source[..], theme::muted())),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(16),
        Constraint::Length(22),
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

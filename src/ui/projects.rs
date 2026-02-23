use super::highlight::highlight_cached;
use crate::app::{App, LoadState};
use crate::model::ProjectHealthStatus;
use crate::theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table, TableState};
use ratatui::Frame;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    if app.projects_drill_active {
        render_drill_down(f, area, app);
    } else {
        render_list(f, area, app);
    }
}

fn health_style(status: &ProjectHealthStatus) -> Style {
    match status {
        ProjectHealthStatus::Healthy => Style::default().fg(theme::GREEN),
        ProjectHealthStatus::Outdated => Style::default().fg(theme::YELLOW),
        ProjectHealthStatus::Missing => Style::default().fg(theme::RED),
        ProjectHealthStatus::NoConfig => theme::muted(),
    }
}

fn render_list(f: &mut Frame, area: Rect, app: &App) {
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
    let count = app.filtered_projects.len();
    let total = app.projects.len();
    let title = if app.search_active && !app.search_query.is_empty() {
        format!(" Projects ({count}/{total}) ")
    } else {
        format!(" Projects ({total}) ")
    };

    let block = Block::default()
        .title(Span::styled(title, theme::title()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::border_focused())
        .style(Style::default().bg(theme::BG));

    if app.projects_state == LoadState::Loading {
        let spinner = app.spinner_char();
        let loading = Paragraph::new(format!("  {spinner} Scanning projects..."))
            .style(theme::muted())
            .block(block);
        f.render_widget(loading, content_area);
        return;
    }

    if app.filtered_projects.is_empty() {
        let msg = if app.search_active && !app.search_query.is_empty() {
            "  No matching projects"
        } else {
            "  No projects found — check ~/.config/misetui/config.toml scan_dirs"
        };
        let empty = Paragraph::new(msg).style(theme::muted()).block(block);
        f.render_widget(empty, content_area);
        return;
    }

    let header = Row::new(vec![
        Cell::from("Name"),
        Cell::from("Path"),
        Cell::from("Tools"),
        Cell::from("Health"),
    ])
    .style(theme::table_header());

    let rows: Vec<Row> = app
        .filtered_projects
        .iter()
        .enumerate()
        .map(|(i, &idx)| {
            let proj = &app.projects[idx];
            let name_hl = app.projects_hl.get(i).map(|v| v.as_slice()).unwrap_or(&[]);

            // Truncate path to last 3 components for readability
            let path_display = {
                let parts: Vec<&str> = proj.path.split('/').collect();
                let shown = &parts[parts.len().saturating_sub(3)..];
                format!("…/{}", shown.join("/"))
            };

            Row::new(vec![
                Cell::from(highlight_cached(&proj.name, name_hl, theme::table_row())),
                Cell::from(Span::styled(path_display, theme::muted())),
                Cell::from(Span::styled(proj.tool_count.to_string(), theme::table_row())),
                Cell::from(Span::styled(proj.health.label(), health_style(&proj.health))),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(20),
        Constraint::Min(20),
        Constraint::Length(6),
        Constraint::Length(12),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(block)
        .row_highlight_style(theme::table_selected());

    let mut state = TableState::default();
    state.select(Some(app.projects_selected));
    f.render_stateful_widget(table, content_area, &mut state);
}

fn render_drill_down(f: &mut Frame, area: Rect, app: &App) {
    // Get the selected project
    let Some(&idx) = app.filtered_projects.get(app.projects_selected) else {
        return;
    };
    let proj = &app.projects[idx];

    let title = format!(" {} — Tool Health ", proj.name);
    let block = Block::default()
        .title(Span::styled(title, theme::title()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::border_focused())
        .style(Style::default().bg(theme::BG));

    if proj.tools.is_empty() {
        let msg = format!("  No tools declared in .mise.toml\n  Path: {}", proj.path);
        let empty = Paragraph::new(msg).style(theme::muted()).block(block);
        f.render_widget(empty, area);
        return;
    }

    let header = Row::new(vec![
        Cell::from("Tool"),
        Cell::from("Required"),
        Cell::from("Installed"),
        Cell::from("Status"),
    ])
    .style(theme::table_header());

    let rows: Vec<Row> = proj
        .tools
        .iter()
        .map(|tool_health| {
            let installed_text = if tool_health.installed.is_empty() {
                "not installed"
            } else {
                &tool_health.installed
            };
            Row::new(vec![
                Cell::from(Span::styled(tool_health.tool.clone(), theme::table_row())),
                Cell::from(Span::styled(tool_health.required.clone(), theme::muted())),
                Cell::from(Span::styled(installed_text.to_owned(), theme::table_row())),
                Cell::from(Span::styled(
                    tool_health.status.label(),
                    health_style(&tool_health.status),
                )),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(16),
        Constraint::Length(14),
        Constraint::Length(14),
        Constraint::Length(12),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(block)
        .row_highlight_style(theme::table_selected());

    let mut state = TableState::default();
    state.select(Some(app.projects_drill_selected));
    f.render_stateful_widget(table, area, &mut state);
}

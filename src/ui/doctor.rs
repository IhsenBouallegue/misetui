use crate::app::{App, LoadState};
use crate::theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};
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

    let filtered = app.visible_doctor_lines();
    let filtered_len = filtered.len();

    let scroll_info = if filtered_len > 0 {
        let pos = app.doctor_scroll.min(filtered_len.saturating_sub(1)) + 1;
        format!(" Doctor ({pos}/{filtered_len}) ")
    } else {
        " Doctor ".to_string()
    };

    let block = Block::default()
        .title(Span::styled(scroll_info, theme::title()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::border_focused())
        .style(ratatui::style::Style::default().bg(theme::BG));

    if app.doctor_state == LoadState::Loading {
        let spinner = app.spinner_char();
        let loading = Paragraph::new(format!("  {spinner} Running mise doctor..."))
            .style(theme::muted())
            .block(block);
        f.render_widget(loading, content_area);
        return;
    }

    if filtered.is_empty() {
        let msg = if app.search_active && !app.search_query.is_empty() {
            "  No matching lines"
        } else {
            "  No output from mise doctor"
        };
        let empty = Paragraph::new(msg).style(theme::muted()).block(block);
        f.render_widget(empty, content_area);
        return;
    }

    let lines: Vec<Line> = filtered
        .iter()
        .map(|line| {
            let style = if line.contains("WARN") || line.contains("ERROR") {
                theme::error()
            } else if line.contains("OK") || line.contains("yes") {
                theme::active_indicator()
            } else if line.starts_with(' ') || line.is_empty() {
                theme::muted()
            } else {
                theme::table_row()
            };
            Line::from(Span::styled(format!("  {line}"), style))
        })
        .collect();

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((app.doctor_scroll as u16, 0));

    f.render_widget(paragraph, content_area);
}

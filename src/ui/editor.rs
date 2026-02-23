use crate::model::{EditorRowStatus, EditorState, EditorTab};
use crate::theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

/// Returns a centered rectangle of fixed width x height within `area`.
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}

/// Styles for row change-status.
fn row_style(status: EditorRowStatus) -> Style {
    match status {
        EditorRowStatus::Unchanged => theme::table_row(),
        EditorRowStatus::Modified => Style::default().fg(theme::YELLOW),
        EditorRowStatus::Added => Style::default().fg(theme::GREEN),
        EditorRowStatus::Deleted => Style::default()
            .fg(Color::Rgb(230, 92, 92))
            .add_modifier(Modifier::CROSSED_OUT),
    }
}

/// Status marker character for the rightmost column.
fn status_marker(status: EditorRowStatus) -> &'static str {
    match status {
        EditorRowStatus::Unchanged => "·",
        EditorRowStatus::Modified => "~",
        EditorRowStatus::Added => "+",
        EditorRowStatus::Deleted => "x",
    }
}

/// Main entry point — renders the editor popup over the existing TUI.
pub fn render_editor(f: &mut Frame, state: &EditorState) {
    let area = centered_rect(72, 26, f.area());
    f.render_widget(Clear, area);

    // Build the title — include "(modified)" when dirty.
    let short_path = shorten_path(&state.file_path, 38);
    let title = if state.dirty {
        format!(" Editor: {short_path} (modified) ")
    } else {
        format!(" Editor: {short_path} ")
    };

    let block = Block::default()
        .title(Span::styled(title, theme::title()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::popup_border())
        .style(theme::popup_bg());

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Outer layout: sub-tab bar | blank | column headers | rows | [edit hint] | bottom hints
    let editing_hint_len = if state.editing { 1u16 } else { 0u16 };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),               // sub-tab bar
            Constraint::Length(1),               // blank
            Constraint::Length(1),               // column headers
            Constraint::Min(1),                  // row list
            Constraint::Length(editing_hint_len), // inline edit hint (conditional)
            Constraint::Length(1),               // bottom hints
        ])
        .split(inner);

    render_tab_bar(f, chunks[0], state);
    render_column_headers(f, chunks[2], state);
    render_rows(f, chunks[3], state);

    if state.editing {
        render_editing_hint(f, chunks[4]);
    }

    render_bottom_hints(f, chunks[5], state);
}

/// Render the "Tools  Env  Tasks" sub-tab bar.
fn render_tab_bar(f: &mut Frame, area: Rect, state: &EditorState) {
    let tab_spans: Vec<Span> = [
        ("  Tools  ", EditorTab::Tools),
        ("Env  ", EditorTab::Env),
        ("Tasks  ", EditorTab::Tasks),
    ]
    .iter()
    .map(|(label, tab)| {
        if *tab == state.tab {
            Span::styled(
                *label,
                Style::default()
                    .fg(theme::RED)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            )
        } else {
            Span::styled(*label, theme::muted())
        }
    })
    .collect();

    f.render_widget(Paragraph::new(Line::from(tab_spans)), area);
}

/// Render the column header row for the active sub-tab.
fn render_column_headers(f: &mut Frame, area: Rect, state: &EditorState) {
    let line = match state.tab {
        EditorTab::Tools => Line::from(vec![
            Span::styled(format!("  {:<24}", "Name"), theme::table_header()),
            Span::styled(format!("{:<16}", "Version"), theme::table_header()),
            Span::styled("St", theme::table_header()),
        ]),
        EditorTab::Env => Line::from(vec![
            Span::styled(format!("  {:<24}", "Key"), theme::table_header()),
            Span::styled(format!("{:<24}", "Value"), theme::table_header()),
            Span::styled("St", theme::table_header()),
        ]),
        EditorTab::Tasks => Line::from(vec![
            Span::styled(format!("  {:<24}", "Name"), theme::table_header()),
            Span::styled(format!("{:<32}", "Command"), theme::table_header()),
            Span::styled("St", theme::table_header()),
        ]),
    };
    f.render_widget(Paragraph::new(line), area);
}

/// Render the list of rows for the active sub-tab.
fn render_rows(f: &mut Frame, area: Rect, state: &EditorState) {
    match state.tab {
        EditorTab::Tools => render_tool_rows(f, area, state),
        EditorTab::Env => render_env_rows(f, area, state),
        EditorTab::Tasks => render_task_rows(f, area, state),
    }
}

fn render_tool_rows(f: &mut Frame, area: Rect, state: &EditorState) {
    if state.tools.is_empty() {
        let empty = Paragraph::new(Span::styled(
            "  (empty — press a to add)",
            theme::muted(),
        ));
        f.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = state
        .tools
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let is_selected_editing = state.editing && i == state.selected;
            let style = row_style(row.status);

            let (name_cell, version_cell) = if is_selected_editing {
                if state.edit_column == 0 {
                    // Editing the name column
                    let name = Line::from(vec![
                        Span::styled("  ", style),
                        Span::styled(state.edit_buffer.clone(), theme::search_input()),
                        Span::styled("█", theme::search_input()),
                        Span::styled(
                            format!("{:<width$}", "", width = 22usize.saturating_sub(state.edit_buffer.len())),
                            style,
                        ),
                        Span::styled(format!("{:<16}", row.version), style),
                        Span::styled(status_marker(row.status), style),
                    ]);
                    return ListItem::new(name);
                } else {
                    // Editing the version column
                    let version_line = Line::from(vec![
                        Span::styled(format!("  {:<24}", row.name), style),
                        Span::styled(state.edit_buffer.clone(), theme::search_input()),
                        Span::styled("█", theme::search_input()),
                        Span::styled(
                            format!("{:<width$}", "", width = 14usize.saturating_sub(state.edit_buffer.len())),
                            style,
                        ),
                        Span::styled(status_marker(row.status), style),
                    ]);
                    return ListItem::new(version_line);
                }
            } else {
                (
                    format!("{:<24}", row.name),
                    format!("{:<16}", row.version),
                )
            };

            let line = Line::from(vec![
                Span::styled(format!("  {name_cell}"), style),
                Span::styled(version_cell, style),
                Span::styled(status_marker(row.status), style),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).highlight_style(theme::table_selected());
    let mut list_state = ListState::default();
    list_state.select(Some(state.selected));
    f.render_stateful_widget(list, area, &mut list_state);
}

fn render_env_rows(f: &mut Frame, area: Rect, state: &EditorState) {
    if state.env_vars.is_empty() {
        let empty = Paragraph::new(Span::styled(
            "  (empty — press a to add)",
            theme::muted(),
        ));
        f.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = state
        .env_vars
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let is_selected_editing = state.editing && i == state.selected;
            let style = row_style(row.status);

            if is_selected_editing {
                if state.edit_column == 0 {
                    let line = Line::from(vec![
                        Span::styled("  ", style),
                        Span::styled(state.edit_buffer.clone(), theme::search_input()),
                        Span::styled("█", theme::search_input()),
                        Span::styled(
                            format!("{:<width$}", "", width = 22usize.saturating_sub(state.edit_buffer.len())),
                            style,
                        ),
                        Span::styled(format!("{:<24}", row.value), style),
                        Span::styled(status_marker(row.status), style),
                    ]);
                    return ListItem::new(line);
                } else {
                    let line = Line::from(vec![
                        Span::styled(format!("  {:<24}", row.key), style),
                        Span::styled(state.edit_buffer.clone(), theme::search_input()),
                        Span::styled("█", theme::search_input()),
                        Span::styled(
                            format!("{:<width$}", "", width = 22usize.saturating_sub(state.edit_buffer.len())),
                            style,
                        ),
                        Span::styled(status_marker(row.status), style),
                    ]);
                    return ListItem::new(line);
                }
            }

            let line = Line::from(vec![
                Span::styled(format!("  {:<24}", row.key), style),
                Span::styled(format!("{:<24}", row.value), style),
                Span::styled(status_marker(row.status), style),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).highlight_style(theme::table_selected());
    let mut list_state = ListState::default();
    list_state.select(Some(state.selected));
    f.render_stateful_widget(list, area, &mut list_state);
}

fn render_task_rows(f: &mut Frame, area: Rect, state: &EditorState) {
    if state.tasks.is_empty() {
        let empty = Paragraph::new(Span::styled(
            "  (empty — press a to add)",
            theme::muted(),
        ));
        f.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = state
        .tasks
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let is_selected_editing = state.editing && i == state.selected;
            let style = row_style(row.status);

            if is_selected_editing {
                if state.edit_column == 0 {
                    let line = Line::from(vec![
                        Span::styled("  ", style),
                        Span::styled(state.edit_buffer.clone(), theme::search_input()),
                        Span::styled("█", theme::search_input()),
                        Span::styled(
                            format!("{:<width$}", "", width = 22usize.saturating_sub(state.edit_buffer.len())),
                            style,
                        ),
                        Span::styled(format!("{:<32}", row.command), style),
                        Span::styled(status_marker(row.status), style),
                    ]);
                    return ListItem::new(line);
                } else {
                    let line = Line::from(vec![
                        Span::styled(format!("  {:<24}", row.name), style),
                        Span::styled(state.edit_buffer.clone(), theme::search_input()),
                        Span::styled("█", theme::search_input()),
                        Span::styled(
                            format!("{:<width$}", "", width = 30usize.saturating_sub(state.edit_buffer.len())),
                            style,
                        ),
                        Span::styled(status_marker(row.status), style),
                    ]);
                    return ListItem::new(line);
                }
            }

            let line = Line::from(vec![
                Span::styled(format!("  {:<24}", row.name), style),
                Span::styled(format!("{:<32}", row.command), style),
                Span::styled(status_marker(row.status), style),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).highlight_style(theme::table_selected());
    let mut list_state = ListState::default();
    list_state.select(Some(state.selected));
    f.render_stateful_widget(list, area, &mut list_state);
}

/// Show "Enter confirm  Esc cancel" hint while editing.
fn render_editing_hint(f: &mut Frame, area: Rect) {
    let line = Line::from(vec![
        Span::styled("  ", theme::muted()),
        Span::styled("Enter", theme::key_hint()),
        Span::styled(" confirm  ", theme::key_desc()),
        Span::styled("Esc", theme::key_hint()),
        Span::styled(" cancel", theme::key_desc()),
    ]);
    f.render_widget(Paragraph::new(line), area);
}

/// Render the bottom hints bar.
fn render_bottom_hints(f: &mut Frame, area: Rect, state: &EditorState) {
    let tab_hint = match state.tab {
        EditorTab::Tools => ("a", "add"),
        EditorTab::Env => ("a", "add env"),
        EditorTab::Tasks => ("a", "add task"),
    };

    let line = Line::from(vec![
        Span::styled(" e ", theme::key_hint()),
        Span::styled("edit  ", theme::key_desc()),
        Span::styled(tab_hint.0, theme::key_hint()),
        Span::styled(format!(" {}  ", tab_hint.1), theme::key_desc()),
        Span::styled("d ", theme::key_hint()),
        Span::styled("delete  ", theme::key_desc()),
        Span::styled("w ", theme::key_hint()),
        Span::styled("write  ", theme::key_desc()),
        Span::styled("h/l ", theme::key_hint()),
        Span::styled("tab  ", theme::key_desc()),
        Span::styled("Esc ", theme::key_hint()),
        Span::styled("close", theme::key_desc()),
    ]);
    f.render_widget(Paragraph::new(line), area);
}

/// Shorten a path string to at most `max_chars` by keeping trailing components.
fn shorten_path(path: &str, max_chars: usize) -> String {
    if path.len() <= max_chars {
        return path.to_string();
    }
    let parts: Vec<&str> = path.split('/').collect();
    let mut result = String::new();
    for part in parts.iter().rev() {
        let candidate = if result.is_empty() {
            part.to_string()
        } else {
            format!("{part}/{result}")
        };
        if candidate.len() > max_chars {
            break;
        }
        result = candidate;
    }
    format!(".../{result}")
}

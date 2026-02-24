use super::highlight::highlight_cached;
use crate::app::{App, LoadState};
use crate::model::EditorRowStatus;
use crate::theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table, TableState};
use ratatui::Frame;
use std::collections::BTreeMap;

fn status_marker(status: EditorRowStatus) -> &'static str {
    match status {
        EditorRowStatus::Unchanged => "",
        EditorRowStatus::Modified => "~",
        EditorRowStatus::Added => "+",
        EditorRowStatus::Deleted => "x",
    }
}

fn status_style(status: EditorRowStatus) -> Style {
    match status {
        EditorRowStatus::Unchanged => theme::table_row(),
        EditorRowStatus::Modified => Style::default().fg(theme::YELLOW),
        EditorRowStatus::Added => Style::default().fg(theme::GREEN),
        EditorRowStatus::Deleted => Style::default()
            .fg(Color::Rgb(230, 92, 92))
            .add_modifier(Modifier::CROSSED_OUT),
    }
}

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
            Span::styled("\u{2588}", theme::search_input()),
        ]))
        .block(search_block);

        f.render_widget(search, chunks[0]);
    }

    let content_area = chunks[1];

    let count = app.filtered_tasks.len();
    let total = app.tasks.len();
    let dirty = app.has_unsaved_editor_changes();
    let title = if app.search_active && !app.search_query.is_empty() {
        format!(" Tasks ({count}/{total}) ")
    } else if dirty {
        format!(" Tasks ({total}) (modified) ")
    } else {
        format!(" Tasks ({total}) ")
    };

    let block = Block::default()
        .title(Span::styled(title, theme::title()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::border_focused())
        .style(Style::default().bg(theme::BG));

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

    // Group tasks by source for section headers when NOT searching
    if !app.search_active && app.editor_states_loaded {
        let mut groups: BTreeMap<&str, Vec<(usize, &crate::model::MiseTask)>> = BTreeMap::new();
        for (i, task) in tasks.iter().enumerate() {
            let key = if task.source.is_empty() { "(runtime)" } else { &task.source };
            groups.entry(key).or_default().push((i, task));
        }

        let mut all_rows: Vec<Row> = Vec::new();
        let mut data_idx_to_visual: Vec<usize> = vec![0; tasks.len()];
        let mut visual_idx = 0;

        for (source, group_tasks) in &groups {
            let source_short = source.rsplit('/').next().unwrap_or(source);
            let header_line = format!("\u{2500}\u{2500} {} \u{2500}\u{2500}", source_short);
            all_rows.push(
                Row::new(vec![
                    Cell::from(Span::styled(header_line, theme::muted())),
                    Cell::from(""),
                    Cell::from(""),
                ])
                .style(Style::default().bg(theme::BG))
            );
            visual_idx += 1;

            for &(data_i, task) in group_tasks {
                data_idx_to_visual[data_i] = visual_idx;
                let overlay = app.editor_task_overlay(&task.source, &task.name);
                let edit_status = overlay.as_ref().map(|(s, _)| *s).unwrap_or(EditorRowStatus::Unchanged);
                let style = status_style(edit_status);
                let marker = status_marker(edit_status);

                let name_cell = Cell::from(Line::from(vec![
                    Span::styled(format!("{marker} "), style),
                    Span::styled(&task.name, style),
                ]));

                let desc_cell = if let Some((_, Some(ref mod_cmd))) = overlay {
                    Cell::from(Span::styled(mod_cmd.clone(), Style::default().fg(theme::YELLOW)))
                } else {
                    Cell::from(Span::styled(task.description.clone(), theme::table_row()))
                };

                let source_short = task.source.rsplit('/').next().unwrap_or(&task.source);
                all_rows.push(Row::new(vec![
                    name_cell, desc_cell,
                    Cell::from(Span::styled(source_short, theme::muted())),
                ]));
                visual_idx += 1;
            }

            // Added rows from EditorState
            let added = app.editor_added_tasks(source);
            for row in &added {
                let style = status_style(EditorRowStatus::Added);
                let name_cell = Cell::from(Line::from(vec![
                    Span::styled("+ ", style),
                    Span::styled(&row.name, style),
                ]));
                let cmd_cell = Cell::from(Span::styled(&row.command, style));
                all_rows.push(Row::new(vec![
                    name_cell, cmd_cell,
                    Cell::from(Span::styled(source.rsplit('/').next().unwrap_or(source), theme::muted())),
                ]));
                visual_idx += 1;
            }
        }

        let visual_selected = data_idx_to_visual.get(app.tasks_selected).copied().unwrap_or(0);

        let widths = [
            Constraint::Length(20),
            Constraint::Min(20),
            Constraint::Length(20),
        ];

        let table = Table::new(all_rows, widths)
            .header(header)
            .block(block)
            .row_highlight_style(theme::table_selected());

        let mut state = TableState::default();
        state.select(Some(visual_selected));
        f.render_stateful_widget(table, content_area, &mut state);
    } else {
        // Flat list (search active or editor not loaded)
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
}

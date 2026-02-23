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

    let count = app.filtered_tools.len();
    let total = app.tools.len();
    let dirty = app.has_unsaved_editor_changes();
    let title = if app.search_active && !app.search_query.is_empty() {
        format!(" Tools ({count}/{total}) ")
    } else if dirty {
        format!(" Tools ({total}) (modified) ")
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

    // Group tools by source for section headers when NOT searching
    if !app.search_active && app.editor_states_loaded {
        let mut groups: BTreeMap<&str, Vec<(usize, &crate::model::InstalledTool)>> = BTreeMap::new();
        for (i, tool) in tools.iter().enumerate() {
            let key = if tool.source.is_empty() { "(runtime)" } else { &tool.source };
            groups.entry(key).or_default().push((i, tool));
        }

        let mut all_rows: Vec<Row> = Vec::new();
        let mut data_idx_to_visual: Vec<usize> = vec![0; tools.len()];
        let mut visual_idx = 0;

        for (source, group_tools) in &groups {
            // Section header row
            let source_short = source.rsplit('/').next().unwrap_or(source);
            let header_line = format!("\u{2500}\u{2500} {} \u{2500}\u{2500}", source_short);
            all_rows.push(
                Row::new(vec![
                    Cell::from(Span::styled(header_line, theme::muted())),
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(""),
                ])
                .style(Style::default().bg(theme::BG))
            );
            visual_idx += 1;

            for &(data_i, tool) in group_tools {
                data_idx_to_visual[data_i] = visual_idx;
                let overlay = app.editor_tool_overlay(&tool.source, &tool.name);
                let edit_status = overlay.as_ref().map(|(s, _)| *s).unwrap_or(EditorRowStatus::Unchanged);
                let style = status_style(edit_status);
                let marker = status_marker(edit_status);

                let status = if tool.active {
                    Cell::from(Span::styled("\u{25cf} active", theme::active_indicator()))
                } else {
                    Cell::from(Span::styled("\u{25cb} inactive", theme::inactive_indicator()))
                };

                // Check for inline editing
                let editing = app.is_editing_tool(&tool.source, &tool.name);

                let name_cell = if let Some(edit) = editing.filter(|e| e.column == 0) {
                    Cell::from(Line::from(vec![
                        Span::styled(format!("{marker} "), style),
                        Span::styled(&edit.buffer, theme::search_input()),
                        Span::styled("\u{2588}", theme::search_input()),
                    ]))
                } else {
                    Cell::from(Line::from(vec![
                        Span::styled(format!("{marker} "), style),
                        Span::styled(&tool.name, style),
                    ]))
                };

                let version_cell = if let Some(edit) = editing.filter(|e| e.column == 1) {
                    Cell::from(Line::from(vec![
                        Span::styled(&edit.buffer, theme::search_input()),
                        Span::styled("\u{2588}", theme::search_input()),
                    ]))
                } else if let Some((_, Some(ref mod_ver))) = overlay {
                    Cell::from(Line::from(vec![
                        Span::styled(mod_ver.clone(), Style::default().fg(theme::YELLOW)),
                    ]))
                } else if let Some(outdated) = app.outdated_map.get(&tool.name) {
                    if outdated.current == tool.version && outdated.latest != tool.version {
                        Cell::from(Line::from(vec![
                            Span::styled(tool.version.clone(), theme::table_row()),
                            Span::styled(format!(" \u{2192} {}", outdated.latest), Style::default().fg(theme::YELLOW)),
                        ]))
                    } else {
                        Cell::from(Span::styled(tool.version.clone(), theme::table_row()))
                    }
                } else {
                    Cell::from(Span::styled(tool.version.clone(), theme::table_row()))
                };

                let source_short = tool.source.rsplit('/').next().unwrap_or(&tool.source);
                all_rows.push(Row::new(vec![
                    name_cell, version_cell, status,
                    Cell::from(Span::styled(source_short, theme::muted())),
                ]));
                visual_idx += 1;
            }

            // Added rows from EditorState (not in CLI data)
            let added = app.editor_added_tools(source);
            for row in &added {
                let style = status_style(EditorRowStatus::Added);
                let name_cell = Cell::from(Line::from(vec![
                    Span::styled("+ ", style),
                    Span::styled(&row.name, style),
                ]));
                let ver_cell = Cell::from(Span::styled(&row.version, style));
                all_rows.push(Row::new(vec![
                    name_cell, ver_cell,
                    Cell::from(Span::styled("+ new", style)),
                    Cell::from(Span::styled(source.rsplit('/').next().unwrap_or(source), theme::muted())),
                ]));
                visual_idx += 1;
            }
        }

        let visual_selected = data_idx_to_visual.get(app.tools_selected).copied().unwrap_or(0);

        let widths = [
            Constraint::Length(18),
            Constraint::Length(22),
            Constraint::Length(12),
            Constraint::Min(10),
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
        let rows: Vec<Row> = tools
            .iter()
            .enumerate()
            .map(|(i, tool)| {
                let status = if tool.active {
                    Cell::from(Span::styled("\u{25cf} active", theme::active_indicator()))
                } else {
                    Cell::from(Span::styled("\u{25cb} inactive", theme::inactive_indicator()))
                };
                let version_cell =
                    if let Some(outdated) = app.outdated_map.get(&tool.name) {
                        if outdated.current == tool.version && outdated.latest != tool.version {
                            Cell::from(Line::from(vec![
                                Span::styled(tool.version.clone(), theme::table_row()),
                                Span::styled(format!(" \u{2192} {}", outdated.latest), Style::default().fg(theme::YELLOW)),
                            ]))
                        } else {
                            Cell::from(Span::styled(tool.version.clone(), theme::table_row()))
                        }
                    } else {
                        Cell::from(Span::styled(tool.version.clone(), theme::table_row()))
                    };
                let name_hl = app.tools_hl.get(i).map(|v| v.as_slice()).unwrap_or(&[]);
                let source_short = tool.source.rsplit('/').next().unwrap_or(&tool.source);
                Row::new(vec![
                    Cell::from(highlight_cached(&tool.name, name_hl, theme::table_row())),
                    version_cell,
                    status,
                    Cell::from(Span::styled(source_short, theme::muted())),
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
}

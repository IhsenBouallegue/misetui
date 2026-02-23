use crate::app::{App, Popup};
use crate::theme;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap,
};
use ratatui::Frame;

pub fn render(f: &mut Frame, app: &App) {
    let Some(popup) = &app.popup else {
        return;
    };

    match popup {
        Popup::VersionPicker {
            tool,
            versions,
            selected,
            use_global,
            search_query,
            filtered_versions,
        } => render_version_picker(
            f,
            tool,
            versions,
            *selected,
            *use_global,
            search_query,
            filtered_versions,
        ),
        Popup::Confirm {
            message,
            action_on_confirm: _,
        } => render_confirm(f, message),
        Popup::Progress { message } => render_progress(f, message, app.spinner_char()),
        Popup::ToolDetail {
            tool_name,
            info,
            scroll,
        } => render_tool_detail(f, tool_name, info, *scroll),
        Popup::Help => render_help(f),
        Popup::ScanConfig { dirs, selected, adding, new_dir, max_depth } => {
            render_scan_config(f, dirs, *selected, *adding, new_dir, *max_depth)
        }
        Popup::Editor(ref _state) => {
            // Full renderer added in Plan 03 (editor.rs)
            let area = centered_rect(70, 24, f.area());
            f.render_widget(Clear, area);
            let block = Block::default()
                .title(Span::styled(" Editor ", theme::title()))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme::popup_border())
                .style(theme::popup_bg());
            let msg = Paragraph::new("  Loading editor...").block(block);
            f.render_widget(msg, area);
        }
    }
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}

fn render_version_picker(
    f: &mut Frame,
    tool: &str,
    versions: &[String],
    selected: usize,
    use_global: bool,
    search_query: &str,
    filtered_versions: &[usize],
) {
    let area = centered_rect(44, 22, f.area());
    f.render_widget(Clear, area);

    let action_label = if use_global { "Use" } else { "Install" };
    let block = Block::default()
        .title(Span::styled(
            format!(" {action_label} {tool} "),
            theme::title(),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::popup_border())
        .style(theme::popup_bg());

    let inner = block.inner(area);

    // Split into search bar + hint + list
    let has_search = !search_query.is_empty();
    let chunks = if has_search {
        Layout::default()
            .constraints([
                Constraint::Length(1), // search
                Constraint::Length(1), // hint
                Constraint::Min(1),   // list
            ])
            .split(inner)
    } else {
        Layout::default()
            .constraints([
                Constraint::Length(0), // no search
                Constraint::Length(1), // hint
                Constraint::Min(1),   // list
            ])
            .split(inner)
    };

    f.render_widget(block, area);

    // Search bar
    if has_search {
        let search = Paragraph::new(Line::from(vec![
            Span::styled(" /", theme::key_hint()),
            Span::styled(search_query, theme::search_input()),
            Span::styled("█", theme::search_input()),
            Span::styled(
                format!(" ({}/{})", filtered_versions.len(), versions.len()),
                theme::muted(),
            ),
        ]));
        f.render_widget(search, chunks[0]);
    }

    let hint = Paragraph::new(Line::from(vec![
        Span::styled(" j/k", theme::key_hint()),
        Span::styled(" select  ", theme::key_desc()),
        Span::styled("Enter", theme::key_hint()),
        Span::styled(" confirm  ", theme::key_desc()),
        Span::styled("Esc", theme::key_hint()),
        Span::styled(" cancel  ", theme::key_desc()),
        Span::styled("type", theme::key_hint()),
        Span::styled(" filter", theme::key_desc()),
    ]));
    f.render_widget(hint, chunks[1]);

    let items: Vec<ListItem> = filtered_versions
        .iter()
        .filter_map(|&i| versions.get(i))
        .map(|v| ListItem::new(Span::styled(format!("  {v}"), theme::table_row())))
        .collect();

    let list = List::new(items).highlight_style(theme::table_selected());
    let mut state = ListState::default();
    state.select(Some(selected));
    f.render_stateful_widget(list, chunks[2], &mut state);
}

fn render_confirm(f: &mut Frame, message: &str) {
    let area = centered_rect(50, 7, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(Span::styled(" Confirm ", theme::title()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::popup_border())
        .style(theme::popup_bg());

    let text = vec![
        Line::default(),
        Line::from(Span::styled(format!("  {message}"), theme::table_row())),
        Line::default(),
        Line::from(vec![
            Span::styled("  Enter", theme::key_hint()),
            Span::styled(" confirm  ", theme::key_desc()),
            Span::styled("Esc", theme::key_hint()),
            Span::styled(" cancel", theme::key_desc()),
        ]),
    ];

    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, area);
}

fn render_progress(f: &mut Frame, message: &str, spinner: char) {
    let area = centered_rect(44, 5, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::popup_border())
        .style(theme::popup_bg());

    let text = vec![
        Line::default(),
        Line::from(Span::styled(
            format!("  {spinner} {message}"),
            theme::progress(),
        )),
    ];

    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, area);
}

fn render_tool_detail(f: &mut Frame, tool_name: &str, info: &str, scroll: usize) {
    let area = centered_rect(60, 22, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(Span::styled(
            format!(" {tool_name} "),
            theme::title(),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::popup_border())
        .style(theme::popup_bg());

    let lines: Vec<Line> = info
        .lines()
        .map(|l| {
            // Highlight JSON keys
            if let Some(colon_pos) = l.find(':') {
                let key = &l[..colon_pos];
                let val = &l[colon_pos..];
                Line::from(vec![
                    Span::styled(format!("  {key}"), theme::title()),
                    Span::styled(val, theme::table_row()),
                ])
            } else {
                Line::from(Span::styled(format!("  {l}"), theme::table_row()))
            }
        })
        .collect();

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((scroll as u16, 0));
    f.render_widget(paragraph, area);
}

fn render_help(f: &mut Frame) {
    let area = centered_rect(54, 26, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(Span::styled(" Help ", theme::title()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::popup_border())
        .style(theme::popup_bg());

    let help_lines = vec![
        "",
        "  Navigation",
        "    j/k ↑/↓     Move up/down",
        "    h/l ←/→     Focus sidebar/content",
        "    Tab          Next tab",
        "    Shift+Tab    Previous tab",
        "    PgUp/PgDn    Scroll by 10",
        "    Mouse scroll Navigate lists",
        "",
        "  Actions",
        "    /            Search (all tabs)",
        "    i            Install (Registry)",
        "    u            Update/Upgrade (Tools/Outdated)",
        "    U            Use global (Registry) / Upgrade all",
        "    d            Uninstall (Tools)",
        "    Enter        Detail (Tools) / Run (Tasks)",
        "    r            Refresh all data",
        "    p            Prune unused versions",
        "    t            Trust config (Config)",
        "    s            Cycle sort column/order",
        "    c            Edit scan config (Projects)",
        "    Esc          Cancel / Close popup",
        "    q            Quit",
        "    ?            This help",
    ];

    let lines: Vec<Line> = help_lines
        .iter()
        .map(|&l| {
            if l.starts_with("  ") && !l.starts_with("    ") {
                Line::from(Span::styled(l, theme::title()))
            } else {
                Line::from(Span::styled(l, theme::table_row()))
            }
        })
        .collect();

    let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);
}

fn render_scan_config(
    f: &mut Frame,
    dirs: &[String],
    selected: usize,
    adding: bool,
    new_dir: &str,
    max_depth: usize,
) {
    // Height: 3 (border+title+depth) + dirs.len() + 1 (add row) + 2 (hints) + padding
    let height = (dirs.len() as u16 + 8).max(12).min(28);
    let area = centered_rect(58, height, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(Span::styled(" Scan Config ", theme::title()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::popup_border())
        .style(theme::popup_bg());

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Layout: depth row | blank | dir list | add row | blank | hints
    let chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(1), // max_depth
            Constraint::Length(1), // blank
            Constraint::Min(1),    // dir list
            Constraint::Length(1), // add row / input
            Constraint::Length(1), // blank
            Constraint::Length(1), // hints
        ])
        .split(inner);

    // Max depth row
    let depth_line = Line::from(vec![
        Span::styled("  max_depth: ", theme::key_desc()),
        Span::styled(max_depth.to_string(), theme::title()),
        Span::styled("   (-/+ to change)", theme::muted()),
    ]);
    f.render_widget(Paragraph::new(depth_line), chunks[0]);

    // Dir list
    let items: Vec<ListItem> = dirs
        .iter()
        .map(|d| ListItem::new(Span::styled(format!("  {d}"), theme::table_row())))
        .collect();

    if items.is_empty() {
        let empty = Paragraph::new(Span::styled(
            "  (no scan dirs — add one with a)",
            theme::muted(),
        ));
        f.render_widget(empty, chunks[2]);
    } else {
        let list = List::new(items)
            .highlight_style(theme::table_selected())
            .highlight_symbol("▶ ");
        let mut state = ListState::default();
        state.select(Some(selected));
        f.render_stateful_widget(list, chunks[2], &mut state);
    }

    // Add row / text input
    if adding {
        let add_row = Line::from(vec![
            Span::styled("  + ", theme::key_hint()),
            Span::styled(new_dir, theme::search_input()),
            Span::styled("█", theme::search_input()),
            Span::styled("  (Enter to add, Esc to cancel)", theme::muted()),
        ]);
        f.render_widget(Paragraph::new(add_row), chunks[3]);
    } else {
        let add_hint = Paragraph::new(Span::styled(
            "  a add dir   d delete   Enter save   Esc cancel",
            theme::muted(),
        ));
        f.render_widget(add_hint, chunks[3]);
    }

    // Bottom hint
    let hint = Line::from(vec![
        Span::styled("  j/k", theme::key_hint()),
        Span::styled(" navigate  ", theme::key_desc()),
        Span::styled("-/+", theme::key_hint()),
        Span::styled(" depth  ", theme::key_desc()),
        Span::styled("Enter", theme::key_hint()),
        Span::styled(" save  ", theme::key_desc()),
        Span::styled("Esc", theme::key_hint()),
        Span::styled(" cancel", theme::key_desc()),
    ]);
    f.render_widget(Paragraph::new(hint), chunks[5]);
}

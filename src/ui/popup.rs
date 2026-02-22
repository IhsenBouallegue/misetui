use crate::app::{App, Popup};
use crate::theme;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};
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
        } => render_version_picker(f, tool, versions, *selected),
        Popup::Confirm {
            message,
            action_on_confirm: _,
        } => render_confirm(f, message),
        Popup::Progress { message } => render_progress(f, message, app.spinner_char()),
        Popup::Help => render_help(f),
    }
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}

fn render_version_picker(f: &mut Frame, tool: &str, versions: &[String], selected: usize) {
    let area = centered_rect(40, 20, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(Span::styled(
            format!(" Install {tool} "),
            theme::title(),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::popup_border())
        .style(theme::popup_bg());

    let inner = block.inner(area);

    // Split into hint + list
    let chunks = Layout::default()
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(inner);

    f.render_widget(block, area);

    let hint = Paragraph::new(Line::from(vec![
        Span::styled(" j/k", theme::key_hint()),
        Span::styled(" select  ", theme::key_desc()),
        Span::styled("Enter", theme::key_hint()),
        Span::styled(" confirm  ", theme::key_desc()),
        Span::styled("Esc", theme::key_hint()),
        Span::styled(" cancel", theme::key_desc()),
    ]));
    f.render_widget(hint, chunks[0]);

    let items: Vec<ListItem> = versions
        .iter()
        .map(|v| ListItem::new(Span::styled(format!("  {v}"), theme::table_row())))
        .collect();

    let list = List::new(items).highlight_style(theme::table_selected());
    let mut state = ListState::default();
    state.select(Some(selected));
    f.render_stateful_widget(list, chunks[1], &mut state);
}

fn render_confirm(f: &mut Frame, message: &str) {
    let area = centered_rect(44, 7, f.area());
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

fn render_help(f: &mut Frame) {
    let area = centered_rect(50, 20, f.area());
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
        "",
        "  Actions",
        "    /            Search (all tabs)",
        "    i            Install (Registry)",
        "    u            Update (Tools)",
        "    d            Uninstall (Tools)",
        "    Enter        Confirm",
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

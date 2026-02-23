use crate::model::{WizardState, WizardStep};
use crate::theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap,
};
use ratatui::Frame;

/// Inline success style — theme::GREEN constant; no success() function in theme.rs.
fn success_style() -> Style {
    Style::default().fg(Color::Rgb(92, 230, 120))
}

/// Centered rect helper (mirrors popup.rs — cannot import from there due to module visibility).
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}

/// Top-level render entry point — dispatches by wizard step.
pub fn render_wizard(f: &mut Frame, wizard: &WizardState) {
    match wizard.step {
        WizardStep::Detecting => render_detecting(f, &wizard.target_dir),
        WizardStep::Review => render_review(f, wizard),
        WizardStep::Preview => render_preview(f, wizard),
        WizardStep::Writing => render_writing(f, &wizard.target_dir),
    }
}

/// Detecting step: centered spinner popup.
fn render_detecting(f: &mut Frame, target: &str) {
    let area = centered_rect(54, 6, f.area());
    f.render_widget(Clear, area);

    let short_target = shorten_path(target, 30);
    let block = Block::default()
        .title(Span::styled(" Bootstrap Wizard ", theme::title()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::popup_border())
        .style(theme::popup_bg());

    let text = vec![
        Line::default(),
        Line::from(Span::styled(
            format!("  Detecting project tools in {short_target}..."),
            theme::progress(),
        )),
        Line::from(Span::styled(
            "  (reading filesystem indicators and legacy pin files)",
            theme::muted(),
        )),
    ];

    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, area);
}

/// Review step: scrollable list of detected tools with toggle support.
fn render_review(f: &mut Frame, wizard: &WizardState) {
    // Height: border(2) + header(2) + tools + agent toggle + blank + hints
    let tool_count = wizard.tools.len().max(1);
    let height = (tool_count as u16 + 9).clamp(14, 30);
    let area = centered_rect(62, height, f.area());
    f.render_widget(Clear, area);

    let short_target = shorten_path(&wizard.target_dir, 28);
    let block = Block::default()
        .title(Span::styled(
            format!(" Bootstrap Wizard — Review ({short_target}) "),
            theme::title(),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::popup_border())
        .style(theme::popup_bg());

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Layout: header | tool list | blank | agent toggle | blank | hints
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header text
            Constraint::Min(1),    // tool list
            Constraint::Length(1), // blank
            Constraint::Length(1), // agent files toggle
            Constraint::Length(1), // blank
            Constraint::Length(1), // hints
        ])
        .split(inner);

    // Header
    let header = Line::from(vec![
        Span::styled("  Tool", theme::table_header()),
        Span::styled("                 Version", theme::table_header()),
        Span::styled("          Source", theme::table_header()),
    ]);
    f.render_widget(Paragraph::new(header), chunks[0]);

    // Tool list
    if wizard.tools.is_empty() {
        let empty = Paragraph::new(Span::styled(
            "  No tools detected — toggle to add manually (feature coming in Phase 4)",
            theme::muted(),
        ));
        f.render_widget(empty, chunks[1]);
    } else {
        let items: Vec<ListItem> = wizard
            .tools
            .iter()
            .map(|t| {
                let check = if t.enabled { "✓" } else { "○" };
                let check_style = if t.enabled {
                    success_style()
                } else {
                    theme::muted()
                };
                let version_display = if t.version.is_empty() {
                    "latest"
                } else {
                    t.version.as_str()
                };
                // Show install status from mise ls -J cross-reference
                let inst_indicator = if t.installed { "↓" } else { " " };
                ListItem::new(Line::from(vec![
                    Span::styled(format!("  {check} "), check_style),
                    Span::styled(format!("{:<16}", t.name), theme::table_row()),
                    Span::styled(format!("{:<16}", version_display), theme::muted()),
                    Span::styled(format!("  {inst_indicator} "), theme::muted()),
                    Span::styled(t.source.as_str(), theme::muted()),
                ]))
            })
            .collect();

        let list = List::new(items).highlight_style(theme::table_selected());
        let mut state = ListState::default();
        state.select(Some(wizard.selected));
        f.render_stateful_widget(list, chunks[1], &mut state);
    }

    // Agent files toggle
    let agent_check = if wizard.write_agent_files { "✓" } else { "○" };
    let agent_style = if wizard.write_agent_files {
        success_style()
    } else {
        theme::muted()
    };
    let agent_toggle = Line::from(vec![
        Span::styled(format!("  {agent_check} "), agent_style),
        Span::styled("Write AGENTS.md + CLAUDE.md", theme::table_row()),
        Span::styled("  (press a to toggle)", theme::muted()),
    ]);
    f.render_widget(Paragraph::new(agent_toggle), chunks[3]);

    // Hints
    let hint = Line::from(vec![
        Span::styled("  j/k", theme::key_hint()),
        Span::styled(" navigate  ", theme::key_desc()),
        Span::styled("Space", theme::key_hint()),
        Span::styled(" toggle tool  ", theme::key_desc()),
        Span::styled("Enter", theme::key_hint()),
        Span::styled(" preview  ", theme::key_desc()),
        Span::styled("Esc", theme::key_hint()),
        Span::styled(" cancel", theme::key_desc()),
    ]);
    f.render_widget(Paragraph::new(hint), chunks[5]);
}

/// Preview step: scrollable generated .mise.toml content.
fn render_preview(f: &mut Frame, wizard: &WizardState) {
    let area = centered_rect(62, 24, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(Span::styled(
            " Bootstrap Wizard — Preview .mise.toml ",
            theme::title(),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::popup_border())
        .style(theme::popup_bg());

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // content
            Constraint::Length(1), // blank
            Constraint::Length(1), // hints
        ])
        .split(inner);

    let lines: Vec<Line> = wizard
        .preview_content
        .lines()
        .map(|l| Line::from(Span::styled(format!("  {l}"), theme::table_row())))
        .collect();

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((wizard.preview_scroll as u16, 0));
    f.render_widget(paragraph, chunks[0]);

    let hint = Line::from(vec![
        Span::styled("  j/k", theme::key_hint()),
        Span::styled(" scroll  ", theme::key_desc()),
        Span::styled("p", theme::key_hint()),
        Span::styled(" back  ", theme::key_desc()),
        Span::styled("Enter", theme::key_hint()),
        Span::styled(" write + install  ", theme::key_desc()),
        Span::styled("Esc", theme::key_hint()),
        Span::styled(" cancel", theme::key_desc()),
    ]);
    f.render_widget(Paragraph::new(hint), chunks[2]);
}

/// Writing step: spinner + message while mise install runs.
fn render_writing(f: &mut Frame, target: &str) {
    let area = centered_rect(54, 6, f.area());
    f.render_widget(Clear, area);

    let short_target = shorten_path(target, 30);
    let block = Block::default()
        .title(Span::styled(" Bootstrap Wizard ", theme::title()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::popup_border())
        .style(theme::popup_bg());

    let text = vec![
        Line::default(),
        Line::from(Span::styled(
            format!("  Writing .mise.toml and running mise install in {short_target}..."),
            theme::progress(),
        )),
    ];

    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, area);
}

/// Truncate a path string to at most `max_chars` by keeping the last N components.
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
    format!("…/{result}")
}

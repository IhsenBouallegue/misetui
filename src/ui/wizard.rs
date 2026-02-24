use crate::app::App;
use crate::model::WizardStep;
use crate::theme;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Frame;

/// Inline success style — theme::GREEN constant; no success() function in theme.rs.
fn success_style() -> Style {
    Style::default().fg(Color::Rgb(92, 230, 120))
}

/// Top-level render entry point — renders in the tab content area.
pub fn render(f: &mut Frame, area: Rect, app: &App) {
    match app.wizard.step {
        WizardStep::Idle => render_idle(f, area, &app.wizard.target_dir),
        WizardStep::Detecting => render_detecting(f, area, &app.wizard.target_dir, app.spinner_char()),
        WizardStep::Review => render_review(f, area, app),
        WizardStep::Preview => render_preview(f, area, app),
        WizardStep::Writing => render_writing(f, area, &app.wizard.target_dir, app.spinner_char()),
    }
}

/// Idle state: welcome screen — "Press Enter to detect project tools"
fn render_idle(f: &mut Frame, area: Rect, target: &str) {
    let block = Block::default()
        .title(Span::styled(" ⚡ Bootstrap Wizard ", theme::title()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::popup_border())
        .style(theme::popup_bg());

    let inner = block.inner(area);
    f.render_widget(block, area);

    let short_target = shorten_path(target, 50);

    // Check if .mise.toml exists
    let mise_toml_path = std::path::Path::new(target).join(".mise.toml");
    let exists_line = if mise_toml_path.exists() {
        Line::from(Span::styled(
            "  .mise.toml already exists — will be overwritten",
            Style::default().fg(Color::Rgb(230, 180, 60)),
        ))
    } else {
        Line::from(Span::styled(
            "  No .mise.toml found in this directory",
            theme::muted(),
        ))
    };

    let text = vec![
        Line::default(),
        Line::from(Span::styled(
            "  Create a .mise.toml for your project",
            theme::table_row(),
        )),
        Line::default(),
        Line::from(vec![
            Span::styled("  Directory: ", theme::muted()),
            Span::styled(short_target, theme::table_row()),
        ]),
        Line::default(),
        exists_line,
        Line::default(),
        Line::default(),
        Line::from(Span::styled(
            "  This wizard will:",
            theme::table_row(),
        )),
        Line::from(Span::styled(
            "    1. Detect tools from project files (package.json, Cargo.toml, etc.)",
            theme::muted(),
        )),
        Line::from(Span::styled(
            "    2. Migrate versions from legacy pin files (.nvmrc, .python-version, etc.)",
            theme::muted(),
        )),
        Line::from(Span::styled(
            "    3. Let you review and toggle detected tools",
            theme::muted(),
        )),
        Line::from(Span::styled(
            "    4. Generate .mise.toml and run mise install",
            theme::muted(),
        )),
        Line::default(),
        Line::default(),
        Line::from(vec![
            Span::styled("  Press ", theme::muted()),
            Span::styled("Enter", theme::key_hint()),
            Span::styled(" to start detection", theme::muted()),
        ]),
    ];

    let paragraph = Paragraph::new(text);
    f.render_widget(paragraph, inner);
}

/// Detecting step: spinner in content area.
fn render_detecting(f: &mut Frame, area: Rect, target: &str, spinner: char) {
    let block = Block::default()
        .title(Span::styled(" ⚡ Bootstrap Wizard ", theme::title()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::popup_border())
        .style(theme::popup_bg());

    let inner = block.inner(area);
    f.render_widget(block, area);

    let short_target = shorten_path(target, 40);
    let text = vec![
        Line::default(),
        Line::from(Span::styled(
            format!("  {spinner} Detecting project tools in {short_target}..."),
            theme::progress(),
        )),
        Line::from(Span::styled(
            "    (reading filesystem indicators and legacy pin files)",
            theme::muted(),
        )),
    ];

    let paragraph = Paragraph::new(text);
    f.render_widget(paragraph, inner);
}

/// Review step: scrollable list of detected tools with toggle support.
fn render_review(f: &mut Frame, area: Rect, app: &App) {
    let wizard = &app.wizard;

    let block = Block::default()
        .title(Span::styled(
            format!(" ⚡ Bootstrap Wizard — Review ({}) ", shorten_path(&wizard.target_dir, 28)),
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
}

/// Preview step: scrollable generated .mise.toml content.
fn render_preview(f: &mut Frame, area: Rect, app: &App) {
    let wizard = &app.wizard;

    let block = Block::default()
        .title(Span::styled(
            " ⚡ Bootstrap Wizard — Preview .mise.toml ",
            theme::title(),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::popup_border())
        .style(theme::popup_bg());

    let inner = block.inner(area);
    f.render_widget(block, area);

    let lines: Vec<Line> = wizard
        .preview_content
        .lines()
        .map(|l| Line::from(Span::styled(format!("  {l}"), theme::table_row())))
        .collect();

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((wizard.preview_scroll as u16, 0));
    f.render_widget(paragraph, inner);
}

/// Writing step: spinner + message while mise install runs.
fn render_writing(f: &mut Frame, area: Rect, target: &str, spinner: char) {
    let block = Block::default()
        .title(Span::styled(" ⚡ Bootstrap Wizard ", theme::title()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::popup_border())
        .style(theme::popup_bg());

    let inner = block.inner(area);
    f.render_widget(block, area);

    let short_target = shorten_path(target, 40);
    let text = vec![
        Line::default(),
        Line::from(Span::styled(
            format!("  {spinner} Writing .mise.toml and running mise install in {short_target}..."),
            theme::progress(),
        )),
    ];

    let paragraph = Paragraph::new(text);
    f.render_widget(paragraph, inner);
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

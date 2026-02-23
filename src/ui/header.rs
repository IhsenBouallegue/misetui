use crate::app::App;
use crate::model::DriftState;
use crate::theme;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::Frame;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let tool_count = app.tools.len();
    let outdated = app.outdated_count();

    let title_spans = vec![
        Span::styled(" misetui ", theme::title()),
        Span::raw("  "),
        Span::styled(format!("Tools: {tool_count}"), theme::header_stat()),
        Span::raw("  "),
        Span::styled(
            format!("Outdated: {outdated}"),
            if outdated > 0 {
                theme::error()
            } else {
                theme::header_stat()
            },
        ),
        Span::raw("  "),
        Span::styled(drift_label(app.drift_state), drift_style(app.drift_state)),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::border_focused())
        .style(ratatui::style::Style::default().bg(theme::BG));

    let header = Paragraph::new(Line::from(title_spans)).block(block);
    f.render_widget(header, area);
}

fn drift_label(state: DriftState) -> &'static str {
    match state {
        DriftState::Checking => " CWD: checking...",
        DriftState::Healthy  => " CWD: healthy",
        DriftState::Drifted  => "! CWD: drifted",
        DriftState::Missing  => "! CWD: missing",
        DriftState::NoConfig => " CWD: no config",
    }
}

fn drift_style(state: DriftState) -> Style {
    match state {
        DriftState::Checking => theme::muted(),
        DriftState::Healthy  => Style::default().fg(theme::GREEN),
        DriftState::Drifted  => Style::default().fg(theme::YELLOW),
        DriftState::Missing  => theme::error(),
        DriftState::NoConfig => theme::muted(),
    }
}

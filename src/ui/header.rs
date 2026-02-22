use crate::app::App;
use crate::theme;
use ratatui::layout::Rect;
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
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::border_focused())
        .style(ratatui::style::Style::default().bg(theme::BG));

    let header = Paragraph::new(Line::from(title_spans)).block(block);
    f.render_widget(header, area);
}

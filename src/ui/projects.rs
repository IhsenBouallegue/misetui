use crate::app::{App, LoadState};
use crate::theme;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    match app.projects_state {
        LoadState::Loading => {
            let spinner = app.spinner_char();
            let msg = format!("  {} Scanning projects...", spinner);
            let block = Block::default()
                .borders(Borders::ALL)
                .title(" Projects ")
                .border_style(ratatui::style::Style::default().fg(theme::MUTED));
            let inner = block.inner(area);
            f.render_widget(block, area);
            let para = Paragraph::new(Line::from(Span::styled(msg, theme::muted())));
            f.render_widget(para, inner);
        }
        LoadState::Loaded => {
            let projects = app.visible_projects();
            let block = Block::default()
                .borders(Borders::ALL)
                .title(format!(" Projects ({}) ", projects.len()))
                .border_style(ratatui::style::Style::default().fg(theme::MUTED));
            let inner = block.inner(area);
            f.render_widget(block, area);

            if projects.is_empty() {
                let para = Paragraph::new(Line::from(Span::styled(
                    "  No projects found. Check ~/.config/misetui/config.toml",
                    theme::muted(),
                )));
                f.render_widget(para, inner);
                return;
            }

            let lines: Vec<Line> = projects
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let selected = i == app.projects_selected;
                    let health_label = p.health.label();
                    let text = format!("  {:<30} {:>12}  {}", p.name, health_label, p.path);
                    let style = if selected {
                        theme::table_selected()
                    } else {
                        ratatui::style::Style::default()
                    };
                    Line::from(Span::styled(text, style))
                })
                .collect();

            let para = Paragraph::new(lines);
            f.render_widget(para, inner);
        }
    }
}

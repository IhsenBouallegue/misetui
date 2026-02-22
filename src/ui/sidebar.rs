use crate::app::{App, Focus, Tab};
use crate::theme;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem};
use ratatui::Frame;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = Tab::ALL
        .iter()
        .map(|t| {
            let style = if *t == app.tab {
                theme::active_tab()
            } else {
                theme::inactive_tab()
            };
            let prefix = if *t == app.tab { " " } else { "  " };
            ListItem::new(Line::from(Span::styled(
                format!("{prefix}{}", t.label()),
                style,
            )))
        })
        .collect();

    let border_style = if app.focus == Focus::Sidebar {
        theme::border_focused()
    } else {
        theme::border()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style)
        .style(ratatui::style::Style::default().bg(theme::BG));

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

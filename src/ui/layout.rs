use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct AppLayout {
    pub header: Rect,
    pub sidebar: Rect,
    pub content: Rect,
    pub footer: Rect,
}

impl AppLayout {
    pub fn new(area: Rect) -> Self {
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // header
                Constraint::Min(5),    // body
                Constraint::Length(2), // footer
            ])
            .split(area);

        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(16), // sidebar
                Constraint::Min(20),   // content
            ])
            .split(vertical[1]);

        Self {
            header: vertical[0],
            sidebar: body[0],
            content: body[1],
            footer: vertical[2],
        }
    }
}

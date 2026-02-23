use ratatui::style::{Color, Modifier, Style};

// Matte Candy / omarchy palette — ANSI-mapped colors
pub const BG: Color = Color::Rgb(6, 12, 16);
pub const FG: Color = Color::Rgb(200, 200, 210);
pub const RED: Color = Color::Rgb(230, 92, 92);
pub const GREEN: Color = Color::Rgb(92, 230, 120);
pub const YELLOW: Color = Color::Rgb(230, 200, 92);
pub const MUTED: Color = Color::Rgb(80, 90, 100);
pub const SURFACE: Color = Color::Rgb(18, 28, 36);
pub const HIGHLIGHT: Color = Color::Rgb(30, 44, 56);

pub fn title() -> Style {
    Style::default().fg(RED).add_modifier(Modifier::BOLD)
}

pub fn header_stat() -> Style {
    Style::default().fg(FG)
}

pub fn active_tab() -> Style {
    Style::default().fg(RED).add_modifier(Modifier::BOLD)
}

pub fn inactive_tab() -> Style {
    Style::default().fg(MUTED)
}

pub fn table_header() -> Style {
    Style::default()
        .fg(RED)
        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
}

pub fn table_row() -> Style {
    Style::default().fg(FG)
}

pub fn table_selected() -> Style {
    Style::default().bg(HIGHLIGHT).fg(FG)
}

pub fn active_indicator() -> Style {
    Style::default().fg(GREEN).add_modifier(Modifier::BOLD)
}

pub fn inactive_indicator() -> Style {
    Style::default().fg(MUTED)
}

pub fn muted() -> Style {
    Style::default().fg(MUTED)
}

pub fn key_hint() -> Style {
    Style::default().fg(RED).add_modifier(Modifier::BOLD)
}

pub fn key_desc() -> Style {
    Style::default().fg(MUTED)
}

pub fn border() -> Style {
    Style::default().fg(MUTED)
}

pub fn border_focused() -> Style {
    Style::default().fg(RED)
}

pub fn search_input() -> Style {
    Style::default().fg(YELLOW)
}

pub fn error() -> Style {
    Style::default().fg(RED).add_modifier(Modifier::BOLD)
}

pub fn popup_border() -> Style {
    Style::default().fg(RED)
}

pub fn popup_bg() -> Style {
    Style::default().bg(SURFACE).fg(FG)
}

pub fn progress() -> Style {
    Style::default().fg(RED)
}

pub fn match_highlight() -> Style {
    Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)
}

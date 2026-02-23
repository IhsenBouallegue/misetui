use crate::app::{App, Tab};
use crate::theme;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let mut hints: Vec<(&str, &str)> = vec![
        ("q", "quit"),
        ("Tab", "switch"),
        ("h/l", "focus"),
        ("j/k", "navigate"),
        ("?", "help"),
        ("type", "to search"),
        ("r", "refresh"),
        ("s", "sort"),
    ];

    match app.tab {
        Tab::Tools => {
            hints.push(("u", "update"));
            hints.push(("d", "uninstall"));
            hints.push(("Enter", "detail"));
        }
        Tab::Registry => {
            hints.push(("i", "install"));
            hints.push(("U", "use global"));
        }
        Tab::Outdated => {
            hints.push(("u", "upgrade"));
            hints.push(("U", "upgrade all"));
        }
        Tab::Tasks => {
            hints.push(("Enter", "run task"));
        }
        Tab::Config => {
            hints.push(("t", "trust"));
        }
        Tab::Environment | Tab::Settings | Tab::Doctor => {}
        Tab::Projects => {
            hints.push(("i", "install tools"));
            hints.push(("u", "upgrade pins"));
            hints.push(("Enter", "drill-down"));
        }
    }

    hints.push(("p", "prune"));

    if app.search_active {
        hints = vec![("Esc", "cancel search"), ("Type", "to filter")];
    }

    let spans: Vec<Span> = hints
        .iter()
        .flat_map(|(key, desc)| {
            vec![
                Span::styled(format!(" {key} "), theme::key_hint()),
                Span::styled(format!("{desc} "), theme::key_desc()),
            ]
        })
        .collect();

    let status_line = if let Some((msg, _)) = &app.status_message {
        Line::from(Span::styled(format!("  {msg}"), theme::muted()))
    } else if app.tab == Tab::Registry {
        // Show selected item description
        if let Some(entry) = app.selected_registry_entry() {
            if let Some(desc) = &entry.description {
                Line::from(Span::styled(format!("  {desc}"), theme::muted()))
            } else {
                Line::default()
            }
        } else {
            Line::default()
        }
    } else {
        Line::default()
    };

    let footer = Paragraph::new(vec![Line::from(spans), status_line])
        .style(ratatui::style::Style::default().bg(theme::BG));
    f.render_widget(footer, area);
}

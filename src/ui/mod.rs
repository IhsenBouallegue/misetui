mod config;
mod doctor;
mod footer;
mod header;
pub mod layout;
mod popup;
mod registry;
mod sidebar;
mod tools;

use crate::app::{App, Tab};
use layout::AppLayout;
use ratatui::Frame;

pub fn render(f: &mut Frame, app: &App) {
    let layout = AppLayout::new(f.area());

    header::render(f, layout.header, app);
    sidebar::render(f, layout.sidebar, app);
    footer::render(f, layout.footer, app);

    match app.tab {
        Tab::Tools => tools::render(f, layout.content, app),
        Tab::Registry => registry::render(f, layout.content, app),
        Tab::Config => config::render(f, layout.content, app),
        Tab::Doctor => doctor::render(f, layout.content, app),
    }

    popup::render(f, app);
}

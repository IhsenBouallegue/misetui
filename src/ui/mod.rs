mod config;
pub(super) mod highlight;
mod doctor;
mod environment;
mod footer;
mod header;
pub mod layout;
mod outdated;
mod popup;
mod projects;
mod registry;
mod settings;
mod sidebar;
mod tasks;
mod tools;
pub(crate) mod wizard;

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
        Tab::Outdated => outdated::render(f, layout.content, app),
        Tab::Registry => registry::render(f, layout.content, app),
        Tab::Tasks => tasks::render(f, layout.content, app),
        Tab::Environment => environment::render(f, layout.content, app),
        Tab::Settings => settings::render(f, layout.content, app),
        Tab::Config => config::render(f, layout.content, app),
        Tab::Doctor => doctor::render(f, layout.content, app),
        Tab::Projects => projects::render(f, layout.content, app),
    }

    popup::render(f, app);
}

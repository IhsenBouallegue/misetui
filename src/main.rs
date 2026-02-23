mod action;
mod app;
mod event;
mod mise;
mod model;
mod theme;
mod tui;
mod ui;

use action::Action;
use app::{App, Popup};
use color_eyre::Result;
use event::EventHandler;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let mut terminal = tui::init()?;

    let (action_tx, mut action_rx) = mpsc::unbounded_channel::<Action>();
    let mut app = App::new(action_tx.clone());
    let mut events = EventHandler::new();

    // Start fetching data
    app.start_fetch();

    loop {
        // Render
        terminal.draw(|f| ui::render(f, &app))?;

        // Wait for next event or action
        tokio::select! {
            Some(event_action) = events.next() => {
                let action = if is_version_picker_active(&app) {
                    remap_version_picker_action(event_action)
                } else if app.search_active && app.popup.is_none() {
                    remap_search_action(event_action)
                } else {
                    remap_normal_action(event_action)
                };
                app.handle_action(action);
            }
            Some(async_action) = action_rx.recv() => {
                app.handle_action(async_action);
            }
        }

        if app.should_quit {
            break;
        }
    }

    tui::restore()?;
    Ok(())
}

fn is_version_picker_active(app: &App) -> bool {
    matches!(app.popup, Some(Popup::VersionPicker { .. }))
}

/// In normal mode, map char inputs to their bound actions
fn remap_normal_action(action: Action) -> Action {
    match action {
        Action::SearchInput(c) => match c {
            'q' => Action::Quit,
            'j' => Action::MoveDown,
            'k' => Action::MoveUp,
            'h' => Action::FocusSidebar,
            'l' => Action::FocusContent,
            '/' => Action::EnterSearch,
            'i' => Action::InstallTool,
            'u' => Action::UpdateTool,
            'd' => Action::UninstallTool,
            '?' => Action::ShowHelp,
            'r' => Action::Refresh,
            'U' => Action::UseTool,
            'p' => Action::PruneTool,
            't' => Action::TrustConfig,
            's' => Action::CycleSortOrder,
            _ => Action::None, // unbound chars do nothing; use / to search
        },
        // Enter is handled contextually in app.rs (popup confirm, tool detail, run task)
        other @ Action::Confirm => other,
        other => other,
    }
}

/// In search mode, let chars and navigation through; remap special keys
fn remap_search_action(action: Action) -> Action {
    match action {
        Action::SearchInput(_) => action,
        Action::Confirm => Action::ExitSearch,
        Action::CancelPopup => Action::CancelPopup,
        Action::SearchBackspace => action,
        // Allow navigating the filtered list while searching
        Action::MoveUp
        | Action::MoveDown
        | Action::PageUp
        | Action::PageDown => action,
        _ => Action::None,
    }
}

/// In version picker mode, route chars to popup search, keep navigation
fn remap_version_picker_action(action: Action) -> Action {
    match action {
        Action::SearchInput(c) => match c {
            'j' => Action::MoveDown,
            'k' => Action::MoveUp,
            _ => Action::PopupSearchInput(c),
        },
        Action::SearchBackspace => Action::PopupSearchBackspace,
        Action::Confirm => Action::Confirm,
        Action::CancelPopup => Action::CancelPopup,
        Action::MoveUp | Action::MoveDown | Action::PageUp | Action::PageDown => action,
        _ => Action::None,
    }
}

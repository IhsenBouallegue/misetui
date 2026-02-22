mod action;
mod app;
mod event;
mod mise;
mod model;
mod theme;
mod tui;
mod ui;

use action::Action;
use app::App;
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
                let action = if app.search_active && app.popup.is_none() {
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
            _ => action, // unbound chars pass through to auto-start search
        },
        other => other,
    }
}

/// In search mode, let chars through as search input, remap special keys
fn remap_search_action(action: Action) -> Action {
    match action {
        Action::SearchInput(_) => action, // all chars pass through
        Action::ConfirmAction => Action::ExitSearch, // Enter exits search
        Action::CancelPopup => Action::CancelPopup,  // Esc exits search
        Action::SearchBackspace => action,
        _ => Action::None, // ignore navigation/actions in search mode
    }
}

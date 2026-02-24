mod action;
mod app;
mod config;
mod event;
mod mise;
mod model;
mod theme;
mod tui;
mod ui;

use action::Action;
use app::{App, Popup, Tab};
use model::WizardStep;
use color_eyre::Result;
use event::EventHandler;
use notify::{Config as NotifyConfig, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc as std_mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;
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

    // Filesystem watcher for drift indicator (DRFT-02)
    // Watches .mise.toml in CWD and ~/.config/mise/config.toml.
    // Uses a std::sync::mpsc channel bridged to tokio via Arc<Mutex<Receiver>>.
    {
        let watch_tx = action_tx.clone();
        tokio::spawn(async move {
            // Build list of paths to watch
            let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let mise_toml = cwd.join(".mise.toml");

            let global_config =
                dirs::config_dir().map(|p| p.join("mise").join("config.toml"));

            // std channel — notify requires a std Sender
            let (std_tx, std_rx) = std_mpsc::channel::<()>();
            let std_rx = Arc::new(Mutex::new(std_rx));

            let mut watcher = match RecommendedWatcher::new(
                move |res: notify::Result<notify::Event>| {
                    if let Ok(event) = res {
                        match event.kind {
                            EventKind::Create(_)
                            | EventKind::Modify(_)
                            | EventKind::Remove(_) => {
                                let _ = std_tx.send(());
                            }
                            _ => {}
                        }
                    }
                },
                NotifyConfig::default()
                    .with_poll_interval(Duration::from_millis(200)),
            ) {
                Ok(w) => w,
                Err(_) => return, // Watcher unavailable — graceful degradation
            };

            // Watch .mise.toml (non-recursive; file may not exist yet — errors silently ignored)
            let _ = watcher.watch(&mise_toml, RecursiveMode::NonRecursive);
            if let Some(ref gc) = global_config {
                let _ = watcher.watch(gc, RecursiveMode::NonRecursive);
            }

            // Debounce loop: coalesce burst writes into one CheckDrift every ~200ms
            loop {
                let std_rx_clone = Arc::clone(&std_rx);
                let received = tokio::task::spawn_blocking(move || {
                    let rx = std_rx_clone.lock().unwrap();
                    rx.recv_timeout(Duration::from_millis(500))
                })
                .await;

                match received {
                    Ok(Ok(())) => {
                        // Drain any additional events accumulated during debounce window
                        tokio::time::sleep(Duration::from_millis(200)).await;
                        {
                            let rx = std_rx.lock().unwrap();
                            while rx.try_recv().is_ok() {}
                        }
                        let _ = watch_tx.send(Action::CheckDrift);
                    }
                    Ok(Err(_)) => {
                        // recv_timeout timed out — just loop (keep watching)
                    }
                    Err(_) => break, // spawn_blocking panicked — exit watcher
                }
            }
        });
    }

    loop {
        // Render
        terminal.draw(|f| ui::render(f, &app))?;

        // Wait for next event or action
        tokio::select! {
            Some(event_action) = events.next() => {
                let action = if is_editor_popup_active(&app) {
                    remap_editor_popup_action(event_action)
                } else if is_version_picker_active(&app) {
                    remap_version_picker_action(event_action)
                } else if is_scan_config_active(&app) {
                    remap_scan_config_action(event_action)
                } else if is_wizard_active(&app) {
                    remap_wizard_action(event_action)
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

fn is_scan_config_active(app: &App) -> bool {
    matches!(app.popup, Some(Popup::ScanConfig { .. }))
}

fn is_wizard_active(app: &App) -> bool {
    app.tab == Tab::Bootstrap && app.wizard.step != WizardStep::Idle
}

fn is_editor_popup_active(app: &App) -> bool {
    matches!(app.popup, Some(Popup::Editor { .. }))
}

/// In wizard popup mode, route chars to wizard navigation actions
fn remap_wizard_action(action: Action) -> Action {
    match action {
        Action::SearchInput(c) => match c {
            'j' => Action::MoveDown,
            'k' => Action::MoveUp,
            ' ' => Action::WizardToggleTool,
            'a' => Action::WizardToggleAgentFiles,
            'n' => Action::WizardNextStep,
            'p' => Action::WizardPrevStep,
            'q' | 'Q' => Action::CancelPopup,
            _ => Action::None,
        },
        Action::Confirm => Action::WizardNextStep,
        Action::CancelPopup => Action::CancelPopup,
        Action::MoveUp | Action::MoveDown | Action::PageUp | Action::PageDown => action,
        _ => Action::None,
    }
}

/// In scan config popup mode, route chars to popup navigation/editing actions
fn remap_scan_config_action(action: Action) -> Action {
    match action {
        Action::SearchInput(c) => match c {
            'j' => Action::MoveDown,
            'k' => Action::MoveUp,
            'd' => Action::UninstallTool,
            'a' => Action::InstallTool,
            '+' => Action::SearchInput('+'),
            '-' => Action::SearchInput('-'),
            'q' | 'Q' => Action::CancelPopup,
            other => Action::SearchInput(other), // for typing in add mode
        },
        other => other,
    }
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
            'd' => Action::EditorDeleteRow,
            'a' => Action::EditorAddRow,
            'v' => Action::ShowToolDetail,
            'w' => Action::EditorWrite,
            '?' => Action::ShowHelp,
            'r' => Action::Refresh,
            'U' => Action::UseTool,
            'p' => Action::PruneTool,
            't' => Action::TrustConfig,
            's' => Action::CycleSortOrder,
            'P' => Action::JumpToDriftProject,
            'c' => Action::OpenScanConfig,
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

/// Editor popup mode — typing into the popup form fields
fn remap_editor_popup_action(action: Action) -> Action {
    match action {
        Action::SearchInput(c) => Action::EditorInput(c),
        Action::SearchBackspace => Action::EditorBackspace,
        Action::Confirm => Action::EditorConfirmEdit,
        Action::CancelPopup => Action::EditorCancelEdit,
        Action::NextTab => Action::EditorNextField,
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

use crate::action::Action;
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers};
use futures::StreamExt;
use std::time::Duration;
use tokio::sync::mpsc;

pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<Action>,
}

impl EventHandler {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        let tick_tx = tx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(250));
            loop {
                interval.tick().await;
                if tick_tx.send(Action::Tick).is_err() {
                    break;
                }
            }
        });

        let render_tx = tx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(16));
            loop {
                interval.tick().await;
                if render_tx.send(Action::Render).is_err() {
                    break;
                }
            }
        });

        let event_tx = tx;
        tokio::spawn(async move {
            let mut reader = EventStream::new();
            loop {
                if let Some(Ok(event)) = reader.next().await {
                    if let Some(action) = map_event(event) {
                        if event_tx.send(action).is_err() {
                            break;
                        }
                    }
                }
            }
        });

        Self { rx }
    }

    pub async fn next(&mut self) -> Option<Action> {
        self.rx.recv().await
    }
}

fn map_event(event: Event) -> Option<Action> {
    match event {
        Event::Key(KeyEvent {
            code, modifiers, ..
        }) => map_key(code, modifiers),
        _ => None,
    }
}

fn map_key(code: KeyCode, modifiers: KeyModifiers) -> Option<Action> {
    if modifiers.contains(KeyModifiers::CONTROL) {
        return match code {
            KeyCode::Char('c') | KeyCode::Char('q') => Some(Action::Quit),
            _ => None,
        };
    }

    match code {
        KeyCode::Down => Some(Action::MoveDown),
        KeyCode::Up => Some(Action::MoveUp),
        KeyCode::Left => Some(Action::FocusSidebar),
        KeyCode::Right => Some(Action::FocusContent),
        KeyCode::Tab => Some(Action::NextTab),
        KeyCode::BackTab => Some(Action::PrevTab),
        KeyCode::PageUp => Some(Action::PageUp),
        KeyCode::PageDown => Some(Action::PageDown),
        KeyCode::Backspace => Some(Action::SearchBackspace),
        KeyCode::Enter => Some(Action::ConfirmAction),
        KeyCode::Esc => Some(Action::CancelPopup),
        KeyCode::Char(c) => Some(Action::SearchInput(c)),
        _ => None,
    }
}

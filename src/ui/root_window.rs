use crossterm::event::KeyCode;
use ratatui::prelude::*;

use crate::action::Action;
use crate::app::App;
use crate::tio::{TerminalEvent, Tio};

use super::chat_sidebar::{self, *};
use super::message_viewer::{self, *};
use super::ActiveUI;

pub struct RootWindow {
    left_session_list: LeftSessionList,
    right_space: RightSpace,
}

impl RootWindow {
    pub fn new() -> Self {
        Self {
            left_session_list: chat_sidebar::LeftSessionList::new(),
            right_space: message_viewer::RightSpace::new(),
        }
    }

    pub fn with_context_model(app: &mut App) -> Self {
        Self {
            left_session_list: LeftSessionList::with_context_model(app),
            right_space: RightSpace::with_context_model(app),
        }
    }

    pub fn handle_base_event(&mut self, event: TerminalEvent, app: &App) -> Action {
        // q to quit, + to add fps, - to reduce fps
        match event {
            TerminalEvent::Error
            | TerminalEvent::Ignore
            | TerminalEvent::Tick
            | TerminalEvent::Mouse(_)
            | TerminalEvent::Resize(_, _) => Action::Nop,
            TerminalEvent::Render => Action::Render,
            TerminalEvent::Key(k) if k.code == KeyCode::Char('q') => Action::Quit,
            // TODO: this event->action map should be put into in the sub ui node left-session-list
            TerminalEvent::Key(k) if k.code == KeyCode::Char('+') => Action::Increment,
            TerminalEvent::Key(k) if k.code == KeyCode::Char('-') => Action::Decrement,
            TerminalEvent::Key(k) if k.code == KeyCode::Tab => match app.active_ui {
                ActiveUI::LEFT => Action::SetActive(ActiveUI::RIGHT),
                ActiveUI::RIGHT => Action::SetActive(ActiveUI::LEFT),
            },
            _ => match app.active_ui {
                ActiveUI::LEFT => self.left_session_list.handle_inner_event(event),
                ActiveUI::RIGHT => self.right_space.handle_inner_event(event),
            },
        }
    }

    pub fn draw(&mut self, app: &mut App, tio: &mut Tio) {
        // TODO: Error handling
        let area = tio.canvas.size().unwrap();
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
            .split(area);
        tio.canvas
            .draw(|f| {
                self.left_session_list.draw(app, f, chunks[0]);
                self.right_space.draw(app, f, chunks[1])
            })
            .unwrap();
    }
}

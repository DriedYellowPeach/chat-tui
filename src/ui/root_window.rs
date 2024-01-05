use crossterm::event::KeyCode;
use ratatui::prelude::*;

use std::rc::Rc;

use crate::{
    action::Action,
    app::App,
    tio::{TerminalEvent, Tio},
};

use super::{chat_sidebar::LeftSessionList, message_viewer::RightSpace, UiId, UiMetaData, UiTag};

#[derive(Default)]
pub struct RootWindow {
    id: UiId,
    tag: Option<UiTag>,
    left_session_list: LeftSessionList,
    right_space: RightSpace,
    pub meta_data: Rc<UiMetaData>,
}

impl RootWindow {
    pub fn with_metadata(self, meta: Rc<UiMetaData>) -> Self {
        let mut ret = self;
        ret.id = meta.next_id();
        ret.meta_data = meta;
        ret
    }

    pub fn with_context_model(self, app: &App) -> Self {
        let mut ret = self;

        ret.left_session_list = ret
            .left_session_list
            .with_metadata(ret.meta_data.clone())
            .with_context_model(app)
            .with_tag(UiTag::ChatSidebar);

        ret.right_space = ret
            .right_space
            .with_metadata(ret.meta_data.clone())
            .with_context_model(app)
            .with_tag(UiTag::MessageViewer);

        ret
    }

    pub fn with_tag(self, tag: UiTag) -> Self {
        let mut ret = self;
        ret.tag = Some(tag);
        ret.meta_data.set_tag(tag, ret.id);
        ret
    }

    pub fn handle_base_event(&mut self, event: TerminalEvent, app: &App) -> Action {
        // q to quit, + to add fps, - to reduce fps
        match event {
            TerminalEvent::Error
            | TerminalEvent::Ignore
            | TerminalEvent::Tick
            | TerminalEvent::Mouse(_)
            | TerminalEvent::Resize(_, _) => Action::Nop,
            TerminalEvent::Render => {
                self.meta_data.set_should_draw(true);
                Action::Nop
            }
            TerminalEvent::Key(k) if k.code == KeyCode::Char('q') => Action::Quit,
            // TODO: this event->action map should be put into in the sub ui node left-session-list
            TerminalEvent::Key(k) if k.code == KeyCode::Char('+') => Action::Increment,
            TerminalEvent::Key(k) if k.code == KeyCode::Char('-') => Action::Decrement,
            TerminalEvent::Key(k) if k.code == KeyCode::Tab => {
                // TODO: error handling
                self.meta_data.next_active();
                Action::Nop
            }
            _ => {
                let active_id = self.meta_data.get_active();
                let left_id = self.meta_data.get_id(&UiTag::ChatSidebar).unwrap();
                let right_id = self.meta_data.get_id(&UiTag::MessageViewer).unwrap();
                if active_id == left_id {
                    self.left_session_list.handle_inner_event(event, app)
                } else if active_id == right_id {
                    self.right_space.handle_inner_event(event, app)
                } else {
                    // do nothing
                    Action::Nop
                }
            }
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

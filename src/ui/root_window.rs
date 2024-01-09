use crossterm::event::KeyCode;
use ratatui::prelude::*;

use std::rc::Rc;

use crate::{
    action::Action,
    app::App,
    tio::{TerminalEvent, Tio},
};

use super::{
    chat_sidebar::LeftSessionList, fps_hint::FpsHint, keypress_hint::KeyPressHint,
    message_viewer::RightSpace, ui_manager::UiManager, UiEntity, UiId, UiMetaData, UiTag,
};

#[derive(Default)]
pub struct RootWindow {
    id: UiId,
    tag: Option<UiTag>,
    left_session_list: LeftSessionList,
    right_space: RightSpace,
    fps_hint: FpsHint,
    key_press_hint: KeyPressHint,
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
        ret.fps_hint = ret.fps_hint.with_metadata(ret.meta_data.clone());
        ret.key_press_hint = ret.key_press_hint.with_metadata(ret.meta_data.clone());

        ret
    }

    pub fn with_tag(self, tag: UiTag) -> Self {
        let mut ret = self;
        ret.tag = Some(tag);
        ret.meta_data.set_tag(tag, ret.id);
        ret
    }

    pub fn handle_base_event(&mut self, event: TerminalEvent, app: &App) -> Action {
        let event = self.key_press_hint.proxy_event(event, app);
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
}

impl UiEntity for RootWindow {
    fn make_blueprints<'a, 'b>(&'a self, area: Rect, ui_mgr: &mut UiManager<'b>, layer: isize)
    where
        'a: 'b,
    {
        let layer1 = layer + 1;
        let layer2 = layer + 2;

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
            .split(area);

        ui_mgr.add_new_blueprint(&self.left_session_list, chunks[0], layer1);
        self.left_session_list
            .make_blueprints(chunks[0], ui_mgr, layer1);

        ui_mgr.add_new_blueprint(&self.right_space, chunks[1], layer1);
        self.right_space.make_blueprints(chunks[1], ui_mgr, layer1);

        ui_mgr.add_new_blueprint(&self.fps_hint, area, layer2);
        self.fps_hint.make_blueprints(area, ui_mgr, layer2);

        ui_mgr.add_new_blueprint(&self.key_press_hint, area, layer2);
        self.key_press_hint.make_blueprints(area, ui_mgr, layer2);
    }
}

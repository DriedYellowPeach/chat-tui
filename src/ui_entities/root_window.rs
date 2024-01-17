use crossterm::event::KeyCode;
use ratatui::prelude::*;

use std::rc::Rc;

use crate::action::{Action, StateModelAction};
use crate::app::App;
use crate::models::state::StateModel;
use crate::tio::{TerminalEvent, Tio};

use super::{
    blueprints::UiBlueprints, chat_sidebar::LeftSessionList, fps_hint::FpsHint,
    keypress_hint::KeyPressHint, message_viewer::RightSpace, TerminalEventResult, UiEntity, UiId,
    UiMetaData, UiTag,
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

    pub fn update_with_context_model(&mut self, app: &App) {
        // do nothing, for current
        self.left_session_list.update_with_context_model(&app);
    }

    pub fn with_tag(self, tag: UiTag) -> Self {
        let mut ret = self;
        ret.tag = Some(tag);
        ret.meta_data.set_tag(tag, ret.id);
        ret
    }
}

impl UiEntity for RootWindow {
    fn make_blueprints<'a, 'b>(&'a self, area: Rect, ui_mgr: &mut UiBlueprints<'b>, layer: isize)
    where
        'a: 'b,
    {
        let layer1 = layer + 1;
        let layer2 = layer + 2;

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
            .split(area);

        self.left_session_list
            .make_blueprints(chunks[0], ui_mgr, layer1);
        ui_mgr.add_new_blueprint(&self.left_session_list, chunks[0], layer1);

        ui_mgr.add_new_blueprint(&self.right_space, chunks[1], layer1);
        self.right_space.make_blueprints(chunks[1], ui_mgr, layer1);

        ui_mgr.add_new_blueprint(&self.fps_hint, area, layer2);
        self.fps_hint.make_blueprints(area, ui_mgr, layer2);

        ui_mgr.add_new_blueprint(&self.key_press_hint, area, layer2);
        self.key_press_hint.make_blueprints(area, ui_mgr, layer2);
    }

    fn handle_terminal_event(&mut self, event: TerminalEvent, app: &App) -> TerminalEventResult {
        let event = self.key_press_hint.handle_terminal_event(event, app);
        let proxied_evt = match event {
            TerminalEventResult::NotHandled(evt) => evt,
            TerminalEventResult::Handled(act) => return TerminalEventResult::Handled(act),
        };

        let sub_ent_evt = match app.state_model {
            StateModel::Chats => self
                .left_session_list
                .handle_terminal_event(proxied_evt, app),
            StateModel::Messages => self.right_space.handle_terminal_event(proxied_evt, app),
            StateModel::FPS => self.fps_hint.handle_terminal_event(proxied_evt, app),
        };
        // there must be best way to not depackage
        let sub_ent_leftover = match sub_ent_evt {
            TerminalEventResult::NotHandled(evt) => evt,
            TerminalEventResult::Handled(act) => return TerminalEventResult::Handled(act),
        };

        match sub_ent_leftover {
            TerminalEvent::Error
            | TerminalEvent::Ignore
            | TerminalEvent::Tick
            | TerminalEvent::Mouse(_)
            | TerminalEvent::Resize(_, _) => TerminalEventResult::Handled(Action::Nop),
            TerminalEvent::Render => {
                self.meta_data.set_should_draw(true);
                TerminalEventResult::Handled(Action::Nop)
            }
            TerminalEvent::Key(k) if k.code == KeyCode::Char('q') => {
                TerminalEventResult::Handled(Action::Quit)
            }
            // TODO: this event->action map should be put into in the sub ui node left-session-list
            TerminalEvent::Key(k) if k.code == KeyCode::Tab => {
                // TODO: error handling
                TerminalEventResult::Handled(Action::StateModel(StateModelAction::NextState))
            }
            _ => TerminalEventResult::Handled(Action::Nop),
        }
    }
}

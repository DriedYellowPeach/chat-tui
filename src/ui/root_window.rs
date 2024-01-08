use crossterm::event::KeyCode;
use ratatui::prelude::*;

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::action::Action;
use crate::app::App;
use crate::tio::TerminalEvent;

use super::{
    chat_sidebar::LeftSessionList, fps_hint::FpsHint, keypress_hint::KeyPressHint,
    message_viewer::RightSpace, TerminalEventResult, UiEntity, UiId, UiMetaData, UiTag,
};

#[derive(Default)]
pub struct RootWindow {
    id: UiId,
    tag: Option<UiTag>,
    left_session_list: Rc<RefCell<LeftSessionList>>,
    right_space: Rc<RefCell<RightSpace>>,
    fps_hint: Rc<RefCell<FpsHint>>,
    key_press_hint: Rc<RefCell<KeyPressHint>>,
    pub meta_data: Rc<UiMetaData>,
    childs_iter: Vec<Weak<RefCell<dyn UiEntity>>>,
    iter_count: usize,
}

impl RootWindow {
    pub fn new(meta: Rc<UiMetaData>, _app: &App) -> Rc<RefCell<Self>> {
        let id = meta.next_id();
        let ret = Self {
            id,
            ..Default::default()
        };

        let ret = Rc::new(RefCell::new(ret));
        let parent = Rc::downgrade(&ret);

        let left = LeftSessionList::new(meta.clone());
        meta.set_entity_tag(
            Rc::downgrade(&(left.clone() as Rc<RefCell<dyn UiEntity>>)),
            UiTag::ChatSidebar,
        );
        left.borrow_mut().with_parent(parent.clone());

        let right = RightSpace::new(meta.clone());
        meta.set_entity_tag(
            Rc::downgrade(&(right.clone() as Rc<RefCell<dyn UiEntity>>)),
            UiTag::MessageViewer,
        );
        right.borrow_mut().with_parent(parent.clone());

        let fps = FpsHint::new(meta.clone());
        fps.borrow_mut().with_parent(parent.clone());

        let key_press = KeyPressHint::new(meta.clone());
        key_press.borrow_mut().with_parent(parent.clone());

        ret.borrow_mut().childs_iter.extend([
            Rc::downgrade(&(left.clone() as Rc<RefCell<dyn UiEntity>>)),
            Rc::downgrade(&(right.clone() as Rc<RefCell<dyn UiEntity>>)),
            Rc::downgrade(&(fps.clone() as Rc<RefCell<dyn UiEntity>>)),
        ]);

        ret.borrow_mut().left_session_list = left;
        ret.borrow_mut().right_space = right;
        ret.borrow_mut().fps_hint = fps;
        ret.borrow_mut().key_press_hint = key_press;

        meta.set_active_entity(Rc::downgrade(
            &(ret.borrow().left_session_list.clone() as Rc<RefCell<dyn UiEntity>>),
        ));
        ret.borrow_mut().meta_data = meta;

        ret
    }

    pub fn proxy_event(&self, event: TerminalEvent) -> TerminalEvent {
        match self
            .key_press_hint
            .borrow_mut()
            .handle_terminal_event(event)
        {
            TerminalEventResult::NotHandled(evt) => evt,
            TerminalEventResult::Handled(_act) => event,
        }
    }
}

// actually RootWindow should not be considered as a UiEntity, it's just a container
impl UiEntity for RootWindow {
    fn handle_terminal_event(&mut self, event: TerminalEvent) -> TerminalEventResult {
        match event {
            TerminalEvent::Tick
            | TerminalEvent::Ignore
            | TerminalEvent::Error
            | TerminalEvent::Mouse(_)
            | TerminalEvent::Resize(_, _) => TerminalEventResult::Handled(Action::Nop),
            TerminalEvent::Render => {
                self.meta_data.set_should_draw(true);
                TerminalEventResult::Handled(Action::Nop)
            }
            TerminalEvent::Key(k) if k.code == KeyCode::Char('q') => {
                TerminalEventResult::Handled(Action::Quit)
            }
            TerminalEvent::Key(k) if k.code == KeyCode::Tab => {
                // TODO: error handling
                if self.left_session_list.borrow().contains_active_entity() {
                    self.meta_data.set_active_entity(Rc::downgrade(
                        &(self.right_space.clone() as Rc<RefCell<dyn UiEntity>>),
                    ));
                } else if self.right_space.borrow().contains_active_entity() {
                    self.meta_data.set_active_entity(Rc::downgrade(
                        &(self.fps_hint.clone() as Rc<RefCell<dyn UiEntity>>),
                    ));
                } else {
                    self.meta_data.set_active_entity(Rc::downgrade(
                        &(self.left_session_list.clone() as Rc<RefCell<dyn UiEntity>>),
                    ));
                }
                TerminalEventResult::Handled(Action::Nop)
            }
            _ => TerminalEventResult::NotHandled(event),
        }
    }

    fn make_blueprints(&self, area: Rect, ui_mgr: &mut super::ui_manager::UiManager) {
        // ui_mgr.add_new_blueprint(Rc::new(RefCell::new(self)), area, 0);
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
            .split(area);

        ui_mgr.add_new_blueprint(self.left_session_list.clone(), chunks[0], 0);
        self.left_session_list
            .borrow_mut()
            .make_blueprints(chunks[0], ui_mgr);

        ui_mgr.add_new_blueprint(self.right_space.clone(), chunks[1], 0);
        self.right_space
            .borrow_mut()
            .make_blueprints(chunks[1], ui_mgr);

        ui_mgr.add_new_blueprint(self.fps_hint.clone(), area, -1);
        self.fps_hint.borrow_mut().make_blueprints(area, ui_mgr);

        ui_mgr.add_new_blueprint(self.key_press_hint.clone(), area, -1);
        self.key_press_hint
            .borrow_mut()
            .make_blueprints(area, ui_mgr);
    }

    fn draw(&mut self, app: &App, frame: &mut Frame, area: Rect) {
        /*do nothing */
    }

    fn get_parent(&self) -> Option<Weak<RefCell<dyn UiEntity>>> {
        None
    }

    fn get_id(&self) -> UiId {
        self.id
    }
}

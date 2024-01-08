use ratatui::layout::Rect;
use ratatui::Frame;

use std::borrow::BorrowMut;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Weak;

use crate::action::Action;
use crate::app::App;
use crate::tio::TerminalEvent;

use self::ui_manager::UiManager;

pub mod chat_sidebar;
pub mod fps_hint;
pub mod input_field;
pub mod keypress_hint;
pub mod message_viewer;
pub mod root_window;
pub mod ui_manager;

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub enum UiTag {
    RootWindow,
    ChatSidebar,
    MessageViewer,
    InputHint,
}

pub type UiId = u16;

pub struct UiMetaData {
    shoud_draw: Cell<bool>,
    current_active: Cell<UiId>,
    // current_active: Cell<Option<Weak<RefCell<dyn UiEntity>>>>,
    id_counter: Cell<u16>,
    draw_counter: Cell<u64>,
    tag_to_id: RefCell<HashMap<UiTag, UiId>>,
    tag_to_entity: RefCell<HashMap<UiTag, Weak<RefCell<dyn UiEntity>>>>,
    active_entity: RefCell<Option<Weak<RefCell<dyn UiEntity>>>>,
}

impl UiMetaData {
    pub fn new() -> Self {
        Self {
            current_active: Cell::new(0),
            shoud_draw: Cell::new(false),
            id_counter: Cell::new(0),
            draw_counter: Cell::new(0),
            tag_to_id: RefCell::new(HashMap::new()),
            tag_to_entity: RefCell::new(HashMap::new()),
            active_entity: RefCell::new(None),
        }
    }

    pub fn next_id(&self) -> UiId {
        let ret = self.id_counter.get();
        self.id_counter.set(ret + 1);
        ret
    }

    pub fn get_active(&self) -> UiId {
        self.current_active.get()
    }

    pub fn get_should_draw(&self) -> bool {
        let it = (0..5).into_iter().cycle();
        self.shoud_draw.get()
    }

    pub fn set_should_draw(&self, should_draw: bool) {
        self.shoud_draw.set(should_draw)
    }

    pub fn next_active(&self) {
        let id = (self.current_active.get() + 1) % self.id_counter.get();
        self.current_active.set(id);
    }

    pub fn set_tag(&self, tag: UiTag, id: UiId) {
        self.tag_to_id.borrow_mut().insert(tag, id);
    }

    pub fn set_active(&self, id: UiId) {
        self.current_active.set(id);
    }

    pub fn get_id(&self, tag: &UiTag) -> Option<UiId> {
        self.tag_to_id.borrow().get(tag).copied()
    }

    pub fn increment_draw_counter(&self) {
        match self.draw_counter.get().checked_add(1) {
            None => self.draw_counter.set(0),
            Some(x) => self.draw_counter.set(x),
        }
    }

    pub fn get_active_entity(&self) -> Option<Weak<RefCell<dyn UiEntity>>> {
        self.active_entity.borrow().clone()
    }

    pub fn set_active_entity(&self, entity: Weak<RefCell<dyn UiEntity>>) {
        *self.active_entity.borrow_mut() = Some(entity);
    }

    pub fn set_active_with_tag(&self, tag: &UiTag) -> Result<(), ()> {
        let Some(entity) = self.tag_to_entity.borrow().get(tag).cloned() else {
            return Err(());
        };
        self.set_active_entity(entity);
        Ok(())
    }

    pub fn set_entity_tag(&self, entity: Weak<RefCell<dyn UiEntity>>, tag: UiTag) {
        self.tag_to_entity.borrow_mut().insert(tag, entity);
    }
}

impl Default for UiMetaData {
    fn default() -> Self {
        Self::new()
    }
}

pub enum TerminalEventResult {
    Handled(Action),
    NotHandled(TerminalEvent),
}

pub trait UiEntity {
    // fn with_context_model(self, app: &App) -> Self;
    // fn with_meta_data(self, meta: Rc<UiMetaData>) -> Self;
    fn handle_terminal_event(&mut self, event: TerminalEvent) -> TerminalEventResult {
        TerminalEventResult::NotHandled(event)
    }
    // update more details in blueprints, the final blueprints will be used to draw the entire UI
    fn make_blueprints(&self, _area: Rect, _ui_mgr: &mut UiManager) {
        /* do nothing */
    }
    // draw will be used by the final blueprints, to draw the UiEntity objects in specific order
    fn draw(&mut self, app: &App, frame: &mut Frame, area: Rect) {
        /* do noting */
    }
    // get parent is used in event-hanlding, if current active can't handle such event, it should pass it
    fn get_parent(&self) -> Option<Weak<RefCell<dyn UiEntity>>>;
    fn get_id(&self) -> UiId;

    fn toggle_highlight(&mut self) {}
}

use std::cell::{Cell, RefCell};
use std::collections::HashMap;

pub mod chat_sidebar;
pub mod fps_hint;
pub mod input_field;
pub mod keypress_hint;
pub mod message_viewer;
pub mod root_window;

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
    id_counter: Cell<u16>,
    draw_counter: Cell<u64>,
    tag_to_id: RefCell<HashMap<UiTag, UiId>>,
}

impl UiMetaData {
    pub fn new() -> Self {
        Self {
            current_active: Cell::new(0),
            shoud_draw: Cell::new(false),
            id_counter: Cell::new(0),
            draw_counter: Cell::new(0),
            tag_to_id: RefCell::new(HashMap::new()),
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
}

impl Default for UiMetaData {
    fn default() -> Self {
        Self::new()
    }
}

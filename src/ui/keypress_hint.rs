// keypress_hint is the designed to show the key press
// It's the first ui entity to handle the keyborad event
// root window should direct the event to it first
// however this entity should not consume the event, it should proxy them

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::{Rc, Weak};
use std::time::Instant;

use crate::app::App;
use crate::tio::TerminalEvent;

use super::{TerminalEventResult, UiEntity, UiId, UiMetaData, UiTag};

pub struct KeyPressHint {
    id: UiId,
    tag: Option<UiTag>,
    meta_data: Rc<UiMetaData>,
    title: String,
    // to let user to see their key press clearly
    // fresh_rate should be very slow, maybe fresh one time in every 3 seconds?
    fresh_rate: f64,
    // should also be able to caculate the current length, this is used to allocate Clear area for
    // widget
    last_tick: Instant,
    cached_key: VecDeque<KeyEvent>,
    max_capacity: usize,
    parent: Option<Weak<RefCell<dyn UiEntity>>>,
}

impl Default for KeyPressHint {
    fn default() -> Self {
        Self {
            id: 0,
            tag: None,
            meta_data: Rc::new(UiMetaData::new()),
            title: String::from("Key Press Hint"),
            fresh_rate: 0.25,
            last_tick: Instant::now(),
            max_capacity: 10,
            cached_key: VecDeque::with_capacity(10),
            parent: None,
        }
    }
}

// so many boilerplate, I should create some base ui object to do this.
// this majority of this entity should be the ui object
impl KeyPressHint {
    pub fn new(meta: Rc<UiMetaData>) -> Rc<RefCell<Self>> {
        let id = meta.next_id();
        let mut ret = Self {
            id,
            ..Default::default()
        };
        ret.meta_data = meta;
        let ret = Rc::new(RefCell::new(ret));
        let _parent = Rc::downgrade(&ret);

        ret
    }

    pub fn with_parent(&mut self, parent: Weak<RefCell<dyn UiEntity>>) -> &mut Self {
        self.parent = Some(parent);
        self
    }

    fn get_ui<'a>(&mut self, key_code_represent: &'a str) -> Paragraph<'a> {
        let keypress_hint = Paragraph::new(key_code_represent).block(
            Block::default()
                .title(self.title.clone())
                .borders(Borders::ALL),
        );
        keypress_hint
    }

    fn add_new_keyevnet(&mut self, event: KeyEvent) {
        if self.cached_key.len() == self.max_capacity {
            self.cached_key.pop_front();
            self.cached_key.push_back(event);
        } else {
            self.cached_key.push_back(event);
        }
    }

    fn is_timeout(&self) -> bool {
        self.last_tick.elapsed().as_secs_f64() > 1.0 / self.fresh_rate
    }

    fn get_string_representatin(&self) -> String {
        let mut ret = String::new();
        for k in self.cached_key.iter() {
            match k.modifiers {
                KeyModifiers::SHIFT => ret.push('󰘶'),
                KeyModifiers::ALT => ret.push('󰘵'),
                KeyModifiers::CONTROL => ret.push('󰘴'),
                _ => {}
            }

            if let KeyCode::Char(c) = k.code {
                ret.push(c);
            }
        }

        ret
    }
}

impl UiEntity for KeyPressHint {
    fn make_blueprints(&self, area: Rect, ui_mgr: &mut super::ui_manager::UiManager) {
        /* do nothing */
    }

    fn draw(&mut self, app: &App, frame: &mut Frame, area: Rect) {
        if self.is_timeout() {
            self.last_tick = Instant::now();
            self.cached_key.clear();
        }
        let representation = self.get_string_representatin();
        let hint_len = std::cmp::max(representation.len() as u16, 17);
        let (x, y) = (area.right(), area.bottom());
        let right_bottom_corner = Rect::new(x - hint_len - 1, y - 3, hint_len, 3);
        frame.render_widget(Clear, right_bottom_corner);
        frame.render_widget(self.get_ui(&representation), right_bottom_corner);
    }

    fn get_parent(&self) -> Option<Weak<RefCell<dyn UiEntity>>> {
        match self.parent {
            Some(ref p) => Some(p.clone()),
            None => None,
        }
    }

    fn handle_terminal_event(&mut self, event: TerminalEvent) -> super::TerminalEventResult {
        let intact = TerminalEventResult::NotHandled(event);
        let TerminalEvent::Key(k_event) = event else {
            return intact;
        };

        self.add_new_keyevnet(k_event);

        intact
    }

    fn get_id(&self) -> UiId {
        self.id
    }
}

// keypress_hint is the designed to show the key press
// It's the first ui entity to handle the keyborad event
// root window should direct the event to it first
// however this entity should not consume the event, it should proxy them

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::time::Instant;

use crate::app::App;
use crate::tio::TerminalEvent;

use super::{TerminalEventResult, UiEntity, UiId, UiMetaData, UiTag};

struct InternalState {
    last_tick: Instant,
    cached_key: VecDeque<KeyEvent>,
}

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
    max_capacity: usize,
    internal_state: RefCell<InternalState>,
}

impl Default for KeyPressHint {
    fn default() -> Self {
        Self {
            id: 0,
            tag: None,
            meta_data: Rc::new(UiMetaData::new()),
            title: String::from("Key Press Hint"),
            fresh_rate: 0.25,
            max_capacity: 10,
            internal_state: RefCell::new(InternalState {
                last_tick: Instant::now(),
                cached_key: VecDeque::with_capacity(10),
            }),
        }
    }
}

// so many boilerplate, I should create some base ui object to do this.
// this majority of this entity should be the ui object
impl KeyPressHint {
    pub fn with_metadata(self, meta: Rc<UiMetaData>) -> Self {
        let mut ret = self;
        ret.id = meta.next_id();
        ret.meta_data = meta;
        ret
    }

    pub fn with_tag(self, tag: UiTag) -> Self {
        let mut ret = self;
        ret.tag = Some(tag);
        ret.meta_data.set_tag(tag, ret.id);
        ret
    }

    fn get_ui<'a>(&self, key_code_represent: &'a str) -> Paragraph<'a> {
        let keypress_hint = Paragraph::new(key_code_represent).block(
            Block::default()
                .title(self.title.clone())
                .borders(Borders::ALL),
        );
        keypress_hint
    }

    fn add_new_keyevnet(&self, event: KeyEvent) {
        let mut internal = self.internal_state.borrow_mut();
        if internal.cached_key.len() == self.max_capacity {
            internal.cached_key.pop_front();
            internal.cached_key.push_back(event);
        } else {
            internal.cached_key.push_back(event);
        }
    }

    fn is_timeout(&self) -> bool {
        self.internal_state
            .borrow()
            .last_tick
            .elapsed()
            .as_secs_f64()
            > 1.0 / self.fresh_rate
    }

    fn get_string_representatin(&self) -> String {
        let mut ret = String::new();
        for k in self.internal_state.borrow().cached_key.iter() {
            match k.modifiers {
                KeyModifiers::SHIFT => ret.push_str("󰘶 "),
                KeyModifiers::ALT => ret.push_str("󰘵 "),
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
    fn draw(&self, _app: &App, frame: &mut Frame, area: Rect) {
        if self.is_timeout() {
            let mut internal = self.internal_state.borrow_mut();
            internal.last_tick = Instant::now();
            internal.cached_key.clear();
        }
        let representation = self.get_string_representatin();
        let hint_len = std::cmp::max(representation.len() as u16, 17);
        let (x, y) = (area.right(), area.bottom());
        let right_bottom_corner = Rect::new(x - hint_len - 1, y - 3, hint_len, 3);
        frame.render_widget(Clear, right_bottom_corner);
        frame.render_widget(self.get_ui(&representation), right_bottom_corner);
    }

    // proxy key event
    fn handle_terminal_event(&mut self, event: TerminalEvent, _app: &App) -> TerminalEventResult {
        let TerminalEvent::Key(k_event) = event else {
            return TerminalEventResult::NotHandled(event);
        };

        self.add_new_keyevnet(k_event);

        TerminalEventResult::NotHandled(event)
    }
}

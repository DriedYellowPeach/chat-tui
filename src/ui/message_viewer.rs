use crossterm::event::KeyCode;
use ratatui::{
    prelude::*,
    text::Line,
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::action::Action;
use crate::app::App;
use crate::models::RemoteData;
use crate::tio::TerminalEvent;

use super::{TerminalEventResult, UiEntity, UiId, UiMetaData, UiTag};

#[derive(Default)]
pub struct RightSpace {
    id: UiId,
    title: String,
    messages: Vec<String>,
    meta_data: Rc<UiMetaData>,
    pub vertical_scroll_state: ScrollbarState,
    pub vertical_scroll: usize,
    pub horizontal_scroll_state: ScrollbarState,
    pub horizontal_scroll: usize,
    parent: Option<Weak<RefCell<dyn UiEntity>>>,
}

impl RightSpace {
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

    pub fn contains_active_entity(&self) -> bool {
        let active = self
            .meta_data
            .get_active_entity()
            .unwrap()
            .upgrade()
            .unwrap();

        let id = active.borrow().get_id();
        self.id == id
    }

    pub fn with_context_model(&mut self, app: &App) -> &mut Self {
        let ret = self;
        let mut messages = Vec::new();
        let title;
        // let default_rs = Self::default();
        if let Some(session_name) = app.messages_model.bind.clone() {
            title = format!("Messages from {:?}", session_name);
            match app.messages_model.get_model_data() {
                RemoteData::Success(data) => data
                    .iter()
                    .for_each(|m| messages.push(format!("[message content of {:?}]\n", m))),
                _ => {
                    messages.push(String::from("messages is loading..."));
                }
            }
        } else {
            title = String::from("No Session Selected");
        }

        let mut long_text = "abcdefg".chars().cycle().take(200).collect::<String>();
        long_text.push('\n');
        let long_text_len = long_text.len();
        messages.push(long_text);

        ret.title = title;
        ret.messages = messages;
        ret.vertical_scroll_state = ret.vertical_scroll_state.content_length(ret.messages.len());
        ret.horizontal_scroll_state = ret.horizontal_scroll_state.content_length(long_text_len);

        ret
    }

    fn update_with_context_model(&mut self, app: &App) {
        let mut messages = Vec::new();
        let title;
        if let Some(session_name) = app.messages_model.bind.clone() {
            title = format!("Messages from {:?}", session_name);
            match app.messages_model.get_model_data() {
                RemoteData::Success(data) => data
                    .iter()
                    .for_each(|m| messages.push(format!("[message content of {:?}]\n", m))),
                _ => {
                    messages.push(String::from("messages is loading..."));
                }
            }
        } else {
            title = String::from("No Session Selected");
        }

        let mut long_text = "abcdefg".chars().cycle().take(200).collect::<String>();
        long_text.push('\n');
        let long_text_len = long_text.len();
        messages.push(long_text);

        self.title = title;
        self.messages = messages;
        self.vertical_scroll_state = self
            .vertical_scroll_state
            .content_length(self.messages.len());
        self.horizontal_scroll_state = self.horizontal_scroll_state.content_length(long_text_len);
    }

    fn get_ui_paragraph<'a>(&self, _app: &App) -> Paragraph<'a> {
        let create_block = |title| {
            Block::default()
                .borders(Borders::ALL)
                .gray()
                .title(Span::styled(
                    title,
                    Style::default().add_modifier(Modifier::BOLD),
                ))
        };
        let text = self
            .messages
            .iter()
            .map(|m| Line::from(m.clone()))
            .collect::<Vec<_>>();

        let paragraph = Paragraph::new(text)
            .gray()
            .block(create_block(self.title.clone()))
            .scroll((self.vertical_scroll as u16, self.horizontal_scroll as u16));

        paragraph
    }

    fn get_ui_horizontal_scrollbar<'a>(&self, _app: &App) -> Scrollbar<'a> {
        let ret = Scrollbar::default()
            .orientation(ScrollbarOrientation::HorizontalBottom)
            .symbols(ratatui::symbols::scrollbar::HORIZONTAL);

        ret
    }

    fn get_ui_vertical_scrollbar<'a>(&self, _app: &App) -> Scrollbar<'a> {
        let ret = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        ret
    }
}

impl UiEntity for RightSpace {
    fn make_blueprints(&self, area: Rect, ui_mgr: &mut super::ui_manager::UiManager) {
        /* do nothing */
    }

    fn draw(&mut self, app: &App, frame: &mut Frame, area: Rect) {
        self.update_with_context_model(app);
        let paragraph = self.get_ui_paragraph(app);
        let vertical_scroll = self.get_ui_vertical_scrollbar(app);
        let horizontal_scroll = self.get_ui_horizontal_scrollbar(app);
        frame.render_widget(paragraph, area);
        frame.render_stateful_widget(vertical_scroll, area, &mut self.vertical_scroll_state);
        frame.render_stateful_widget(horizontal_scroll, area, &mut self.horizontal_scroll_state);
    }

    fn handle_terminal_event(&mut self, event: TerminalEvent) -> super::TerminalEventResult {
        let mut ret = TerminalEventResult::NotHandled(event);
        if let TerminalEvent::Key(key) = event {
            match key.code {
                KeyCode::Char('j') => {
                    self.vertical_scroll = self.vertical_scroll.saturating_add(1);
                    self.vertical_scroll_state =
                        self.vertical_scroll_state.position(self.vertical_scroll);
                    ret = TerminalEventResult::Handled(Action::Nop);
                }
                KeyCode::Char('k') => {
                    self.vertical_scroll = self.vertical_scroll.saturating_sub(1);
                    self.vertical_scroll_state =
                        self.vertical_scroll_state.position(self.vertical_scroll);
                    ret = TerminalEventResult::Handled(Action::Nop);
                }
                KeyCode::Char('h') => {
                    self.horizontal_scroll = self.horizontal_scroll.saturating_sub(1);
                    self.horizontal_scroll_state = self
                        .horizontal_scroll_state
                        .position(self.horizontal_scroll);
                    ret = TerminalEventResult::Handled(Action::Nop);
                }
                KeyCode::Char('l') => {
                    self.horizontal_scroll = self.horizontal_scroll.saturating_add(1);
                    self.horizontal_scroll_state = self
                        .horizontal_scroll_state
                        .position(self.horizontal_scroll);
                    ret = TerminalEventResult::Handled(Action::Nop);
                }
                _ => {}
            }
        };

        ret
    }

    fn get_parent(&self) -> Option<Weak<std::cell::RefCell<dyn UiEntity>>> {
        match self.parent {
            Some(ref p) => Some(p.clone()),
            None => None,
        }
    }

    fn get_id(&self) -> UiId {
        self.id
    }
}

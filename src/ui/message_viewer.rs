use crossterm::event::KeyCode;
use ratatui::prelude::*;
use ratatui::text::Line;
use ratatui::widgets::{
    Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
};

use std::cell::RefCell;
use std::rc::Rc;

use crate::action::Action;
use crate::app::App;
use crate::models::RemoteData;
use crate::tio::TerminalEvent;

use super::{TerminalEventResult, UiEntity, UiId, UiMetaData, UiTag};

#[derive(Default)]
struct InternalState {
    messages: Vec<String>,
    title: String,
    pub vertical_scroll_state: ScrollbarState,
    pub vertical_scroll: usize,
    pub horizontal_scroll_state: ScrollbarState,
    pub horizontal_scroll: usize,
}

#[derive(Default)]
pub struct RightSpace {
    id: UiId,
    tag: Option<UiTag>,
    meta_data: Rc<UiMetaData>,
    internal_state: RefCell<InternalState>,
}

impl RightSpace {
    pub fn with_metadata(self, meta: Rc<UiMetaData>) -> Self {
        let mut ret = self;
        ret.id = meta.next_id();
        ret.meta_data = meta;
        ret
    }

    pub fn with_context_model(self, app: &App) -> Self {
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

        let mut internal = ret.internal_state.borrow_mut();
        internal.title = title;
        internal.messages = messages;
        internal.vertical_scroll_state = internal
            .vertical_scroll_state
            .content_length(internal.messages.len());
        internal.horizontal_scroll_state = internal
            .horizontal_scroll_state
            .content_length(long_text_len);
        drop(internal);

        ret
    }

    pub fn with_tag(self, tag: UiTag) -> Self {
        let mut ret = self;
        ret.tag = Some(tag);
        ret.meta_data.set_tag(tag, ret.id);
        ret
    }

    fn update_with_context_model(&self, app: &App) {
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

        let mut internal = self.internal_state.borrow_mut();
        internal.title = title;
        internal.messages = messages;
        internal.vertical_scroll_state = internal
            .vertical_scroll_state
            .content_length(internal.messages.len());
        internal.horizontal_scroll_state = internal
            .horizontal_scroll_state
            .content_length(long_text_len);
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
        let internal = self.internal_state.borrow();
        let text = internal
            .messages
            .iter()
            .map(|m| Line::from(m.clone()))
            .collect::<Vec<_>>();

        let paragraph = Paragraph::new(text)
            .gray()
            .block(create_block(internal.title.clone()))
            .scroll((
                internal.vertical_scroll as u16,
                internal.horizontal_scroll as u16,
            ));

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
    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
        self.update_with_context_model(app);
        let paragraph = self.get_ui_paragraph(app);
        let vertical_scroll = self.get_ui_vertical_scrollbar(app);
        let horizontal_scroll = self.get_ui_horizontal_scrollbar(app);
        frame.render_widget(paragraph, area);
        frame.render_stateful_widget(
            vertical_scroll,
            area,
            &mut self.internal_state.borrow_mut().vertical_scroll_state,
        );
        frame.render_stateful_widget(
            horizontal_scroll,
            area,
            &mut self.internal_state.borrow_mut().horizontal_scroll_state,
        );
    }

    fn handle_terminal_event(&mut self, event: TerminalEvent, _app: &App) -> TerminalEventResult {
        let mut internal = self.internal_state.borrow_mut();
        let mut ret = TerminalEventResult::Handled(Action::Nop);
        match event {
            TerminalEvent::Key(key) => match key.code {
                KeyCode::Char('j') => {
                    internal.vertical_scroll = internal.vertical_scroll.saturating_add(1);
                    internal.vertical_scroll_state = internal
                        .vertical_scroll_state
                        .position(internal.vertical_scroll);
                }
                KeyCode::Char('k') => {
                    internal.vertical_scroll = internal.vertical_scroll.saturating_sub(1);
                    internal.vertical_scroll_state = internal
                        .vertical_scroll_state
                        .position(internal.vertical_scroll);
                }
                KeyCode::Char('h') => {
                    internal.horizontal_scroll = internal.horizontal_scroll.saturating_sub(1);
                    internal.horizontal_scroll_state = internal
                        .horizontal_scroll_state
                        .position(internal.horizontal_scroll);
                }
                KeyCode::Char('l') => {
                    internal.horizontal_scroll = internal.horizontal_scroll.saturating_add(1);
                    internal.horizontal_scroll_state = internal
                        .horizontal_scroll_state
                        .position(internal.horizontal_scroll);
                }
                _ => ret = TerminalEventResult::NotHandled(event),
            },
            _ => ret = TerminalEventResult::NotHandled(event),
        }

        ret
    }
}

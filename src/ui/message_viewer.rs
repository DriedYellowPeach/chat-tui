use crossterm::event::KeyCode;
use ratatui::{
    prelude::*,
    text::Line,
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

use crate::action::Action;
use crate::app::App;
use crate::model::RemoteData;
use crate::tio::TerminalEvent;

#[derive(Default)]
pub struct RightSpace {
    title: String,
    messages: Vec<String>,
    pub vertical_scroll_state: ScrollbarState,
    pub vertical_scroll: usize,
    pub horizontal_scroll_state: ScrollbarState,
    pub horizontal_scroll: usize,
}

impl RightSpace {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_context_model(app: &mut App) -> Self {
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

        let mut ret = Self {
            title,
            messages,
            ..Default::default()
        };

        ret.vertical_scroll_state = ret.vertical_scroll_state.content_length(ret.messages.len());
        ret.horizontal_scroll_state = ret.horizontal_scroll_state.content_length(long_text_len);
        ret
    }

    fn update_with_context_model(&mut self, app: &mut App) {
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

        self.title = title;
        self.messages = messages;
        self.vertical_scroll_state = self
            .vertical_scroll_state
            .content_length(self.messages.len());
        self.horizontal_scroll_state = self.horizontal_scroll_state.content_length(long_text_len);
    }

    fn get_ui_paragraph<'a>(&self, _app: &mut App) -> Paragraph<'a> {
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

    fn get_ui_horizontal_scrollbar<'a>(&self, _app: &mut App) -> Scrollbar<'a> {
        let ret = Scrollbar::default()
            .orientation(ScrollbarOrientation::HorizontalBottom)
            .symbols(ratatui::symbols::scrollbar::HORIZONTAL);

        ret
    }

    fn get_ui_vertical_scrollbar<'a>(&self, _app: &mut App) -> Scrollbar<'a> {
        let ret = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        ret
    }

    pub fn draw(&mut self, app: &mut App, frame: &mut Frame<'_>, area: Rect) {
        self.update_with_context_model(app);
        let paragraph = self.get_ui_paragraph(app);
        let vertical_scroll = self.get_ui_vertical_scrollbar(app);
        let horizontal_scroll = self.get_ui_horizontal_scrollbar(app);
        frame.render_widget(paragraph, area);
        frame.render_stateful_widget(vertical_scroll, area, &mut self.vertical_scroll_state);
        frame.render_stateful_widget(horizontal_scroll, area, &mut self.horizontal_scroll_state);
    }

    pub fn handle_inner_event(&mut self, event: TerminalEvent) -> Action {
        if let TerminalEvent::Key(key) = event {
            match key.code {
                KeyCode::Char('j') => {
                    self.vertical_scroll = self.vertical_scroll.saturating_add(1);
                    self.vertical_scroll_state =
                        self.vertical_scroll_state.position(self.vertical_scroll);
                }
                KeyCode::Char('k') => {
                    self.vertical_scroll = self.vertical_scroll.saturating_sub(1);
                    self.vertical_scroll_state =
                        self.vertical_scroll_state.position(self.vertical_scroll);
                }
                KeyCode::Char('h') => {
                    self.horizontal_scroll = self.horizontal_scroll.saturating_sub(1);
                    self.horizontal_scroll_state = self
                        .horizontal_scroll_state
                        .position(self.horizontal_scroll);
                }
                KeyCode::Char('l') => {
                    self.horizontal_scroll = self.horizontal_scroll.saturating_add(1);
                    self.horizontal_scroll_state = self
                        .horizontal_scroll_state
                        .position(self.horizontal_scroll);
                }
                _ => {}
            }
        }

        Action::Nop
    }
}

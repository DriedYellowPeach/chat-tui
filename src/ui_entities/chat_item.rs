use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use std::cell::RefCell;

use crate::app::App;
use crate::models::ChatSession;

use super::UiEntity;

pub struct ChatItem {
    is_highlight: bool,
    pub id: ChatSession,
    internal: RefCell<InternalState>,
}

struct InternalState {
    name: String,
    msg_preview: String,
    msg_timestamp: String,
    unread_msg: u32,
}

struct ItemWidget<'a> {
    name_sec: Paragraph<'a>,
    msg_preview_sec: Paragraph<'a>,
    msg_timestamp_sec: Paragraph<'a>,
    unread_msg_sec: Paragraph<'a>,
}

impl ChatItem {
    pub fn new(id: ChatSession) -> Self {
        Self {
            id,
            is_highlight: false,
            internal: RefCell::new(InternalState {
                name: String::new(),
                msg_preview: String::new(),
                msg_timestamp: String::new(),
                unread_msg: 0,
            }),
        }
    }

    fn with_highlight(self) -> Self {
        let mut ret = self;
        ret.is_highlight = true;
        ret
    }

    fn update_with_context_model(&self, _app: &App) {
        let mut internal = self.internal.borrow_mut();
        match self.id {
            ChatSession::Group(ref _gid) => {
                internal.name = "Nodic Nostalgia".to_string();
                internal.unread_msg = 23;
                internal.msg_preview =
                    "Neil: Welcome, Everybody. Especially you, Mr.Gump.".to_string();
                internal.msg_timestamp = "Just Now".to_string();
            }
            ChatSession::WithOther(ref _uid) => {
                internal.name = "❤Raphina❤".to_string();
                internal.unread_msg = 5;
                internal.msg_preview = "Raphina: Dinner together?".to_string();
                internal.msg_timestamp = "5m ago".to_string();
            }
        }
    }

    fn get_ui<'a>(&self, app: &App, _area: Rect) -> ItemWidget<'a> {
        self.update_with_context_model(app);

        let internal = self.internal.borrow();

        let name = if self.is_highlight {
            format!("*{}", internal.name)
        } else {
            internal.name.clone()
        };

        let name_sec = Paragraph::new(name)
            .block(Block::default().borders(Borders::NONE))
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Left)
            .wrap(ratatui::widgets::Wrap { trim: true });

        let text: Text = internal.unread_msg.to_string().fg(Color::Red).into();

        let unread_msg_sec = Paragraph::new(text)
            .block(Block::default().borders(Borders::NONE))
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Right)
            .wrap(ratatui::widgets::Wrap { trim: true });

        let msg_preview_sec = Paragraph::new(internal.msg_preview.clone())
            .block(Block::default().borders(Borders::NONE))
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Left)
            .wrap(ratatui::widgets::Wrap { trim: true });

        let msg_timestamp_sec = Paragraph::new(internal.msg_timestamp.clone())
            .block(Block::default().borders(Borders::NONE))
            .style(Style::default().fg(Color::White))
            // .style(Style::default().fg(Color::White).bg(Color::Red))
            .alignment(Alignment::Right)
            .wrap(ratatui::widgets::Wrap { trim: true });

        ItemWidget {
            name_sec,
            msg_preview_sec,
            msg_timestamp_sec,
            unread_msg_sec,
        }
    }
}

impl UiEntity for ChatItem {
    fn draw(&self, app: &App, frame: &mut ratatui::prelude::Frame, area: Rect) {
        let item = self.get_ui(app, area);

        let top_bottom_sep = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Min(1),
                Constraint::Min(1),
            ])
            .split(area);

        let left_right_top = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(80), Constraint::Min(5)])
            .split(top_bottom_sep[0].inner(&Margin {
                horizontal: 1,
                vertical: 0,
            }));

        let left_right_bottom = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(80), Constraint::Min(10)])
            .split(top_bottom_sep[1].inner(&Margin {
                horizontal: 1,
                vertical: 0,
            }));

        let name_sec = left_right_top[0];
        let unread_sec = left_right_top[1];
        let msg_preview_sec = left_right_bottom[0];
        let time_sec = left_right_bottom[1];
        let separator = top_bottom_sep[2];

        frame.render_widget(item.name_sec, name_sec);
        frame.render_widget(item.msg_preview_sec, msg_preview_sec);
        frame.render_widget(item.unread_msg_sec, unread_sec);
        frame.render_widget(item.msg_timestamp_sec, time_sec);
        frame.render_widget(
            Paragraph::new("-".repeat(separator.width as usize)),
            separator,
        );
    }
}

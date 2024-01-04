use crossterm::event::KeyCode;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};

use crate::{
    action::{Action, MessagesModelAction},
    app::App,
    model::{ChatSession, RemoteData, SessionRecord},
    tio::TerminalEvent,
};

use super::ActiveUI;

struct Content {
    chat_session: ChatSession,
    session_name: String,
    last_msg: String,
    last_msg_time: String,
    unread_msg: usize,
}

impl From<&SessionRecord> for Content {
    fn from(value: &SessionRecord) -> Self {
        let chat_session = value.session.clone();
        match &value.session {
            ChatSession::Group(gid) => Self {
                chat_session,
                session_name: format!("g/{:?}", gid),
                last_msg: format!("content from {:?}", value.bookmark),
                last_msg_time: format!("time from {:?}", value.bookmark),
                unread_msg: value.unread_msg,
            },
            ChatSession::WithOther(uid) => Self {
                chat_session,
                session_name: format!("u/{:?}", uid),
                last_msg: format!("content from {:?}", value.bookmark),
                last_msg_time: format!("time from {:?}", value.bookmark),
                unread_msg: value.unread_msg,
            },
        }
    }
}

pub struct LeftSessionList {
    state: ListState,
    items: Vec<Content>,
}

enum SessionListUI<'a> {
    Already(List<'a>),
    Waiting(List<'a>),
}

impl LeftSessionList {
    pub fn new() -> Self {
        Self {
            state: ListState::default(),
            items: Vec::new(),
        }
    }

    pub fn with_context_model(app: &mut App) -> Self {
        let mut items = Vec::new();
        match app.sessions_model.get_model_data() {
            RemoteData::Success(data) => {
                data.0
                    .iter()
                    .for_each(|record| items.push(Content::from(record)));
                assert_ne!(items.len(), 0);
            }
            _ => {
                assert_eq!(items.len(), 0);
            }
        }
        Self {
            state: ListState::default(),
            items,
        }
    }

    fn update_with_context_model(&mut self, app: &mut App) {
        let mut items = Vec::new();
        match app.sessions_model.get_model_data() {
            RemoteData::Success(data) => {
                data.0
                    .iter()
                    .for_each(|record| items.push(Content::from(record)));
                assert_ne!(items.len(), 0);
            }
            _ => {
                assert_eq!(items.len(), 0);
            }
        }
        self.items = items;
    }

    fn get_ui<'a>(&mut self, app: &mut App, area: Rect) -> SessionListUI<'a> {
        self.update_with_context_model(app);
        let mut title = "Chats";
        let mut items: Vec<ListItem> = self
            .items
            .iter()
            .map(|content| {
                let lines = vec![
                    // Line::from("═".repeat(area.width as usize).fg(Color::White)),
                    Line::from(content.session_name.clone().bold()),
                    Line::from(content.last_msg_time.clone().fg(Color::Green)),
                    Line::from(content.last_msg.clone().fg(Color::Blue)),
                    // Line::from(content.unread_msg.to_string().fg(Color::Red)),
                    Line::from(vec![
                        "unread: ".fg(Color::White),
                        content
                            .unread_msg
                            .to_string()
                            .bg(Color::Red)
                            .fg(Color::Black),
                    ]),
                    Line::from("═".repeat(area.width as usize).fg(Color::White)),
                ];
                ListItem::new(lines)
            })
            .collect();

        if items.is_empty() {
            // let paragraph = Paragraph::new("Waiting for data...")
            //     .block(Block::default().borders(Borders::ALL).title("Sessions"));
            // return SessionListUI::Waiting(paragraph);
            title = "Chats";
            items.push(ListItem::new(Line::from("Waiting for data")));
        }

        // let my_list = vec!["hello", "hello", "hello"];
        // let items: Vec<ListItem> = my_list
        //     .iter()
        //     .map(|content| {
        //         ListItem::new(*content)
        //             .style(Style::default().fg(Color::Black).bg(Color::White))
        //     })
        //     .collect();

        SessionListUI::Already(
            List::new(items)
                .block(Block::default().borders(Borders::ALL).title(title))
                .highlight_style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                ),
        )

        // f.render_stateful_widget(list, area, &mut self.state);
    }

    pub fn draw(&mut self, app: &mut App, frame: &mut Frame, area: Rect) {
        let ui = self.get_ui(app, area);
        match ui {
            SessionListUI::Already(list) => {
                frame.render_stateful_widget(list, area, &mut self.state);
            }
            SessionListUI::Waiting(list) => {
                frame.render_stateful_widget(list, area, &mut self.state)
            }
        }
    }

    pub fn handle_inner_event(&mut self, event: TerminalEvent) -> Action {
        match event {
            TerminalEvent::Key(k) if k.code == KeyCode::Char('j') => {
                self.next();
                Action::Nop
            }
            TerminalEvent::Key(k) if k.code == KeyCode::Char('k') => {
                self.prev();
                Action::Nop
            }
            // TerminalEvent::Key(k) if k.code == KeyCode::Char('r') => Action::ReloadSessionPool,
            TerminalEvent::Key(k) if k.code == KeyCode::Enter => {
                if let Some(offset) = self.state.selected() {
                    let actions = vec![
                        Action::MessagesModel(MessagesModelAction::SetBind(
                            self.items[offset].chat_session.clone(),
                        )),
                        Action::SetActive(ActiveUI::RIGHT),
                    ];
                    Action::MultiAction(actions)
                } else {
                    Action::Nop
                }
            }
            _ => Action::Nop,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn prev(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }
}

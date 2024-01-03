use crossterm::event::KeyCode;
use ratatui::prelude::*;

use crate::action::Action;
use crate::app::App;
use crate::tio::{TerminalEvent, Tio};

use left_session_list::*;
use right_space::RightSpace;

pub enum ActiveUI {
    LEFT,
    RIGHT,
}

pub struct UITree {
    left_session_list: LeftSessionList,
    right_space: RightSpace,
}

impl UITree {
    pub fn new() -> Self {
        Self {
            left_session_list: left_session_list::LeftSessionList::new(),
            right_space: right_space::RightSpace::new(),
        }
    }

    pub fn with_context_model(app: &mut App) -> Self {
        Self {
            left_session_list: LeftSessionList::with_context_model(app),
            right_space: RightSpace::with_context_model(app),
        }
    }

    pub fn handle_base_event(&mut self, event: TerminalEvent, app: &App) -> Action {
        // q to quit, + to add fps, - to reduce fps
        match event {
            TerminalEvent::Error
            | TerminalEvent::Ignore
            | TerminalEvent::Tick
            | TerminalEvent::Mouse(_)
            | TerminalEvent::Resize(_, _) => Action::Nop,
            TerminalEvent::Render => Action::Render,
            TerminalEvent::Key(k) if k.code == KeyCode::Char('q') => Action::Quit,
            // TODO: this event->action map should be put into in the sub ui node left-session-list
            TerminalEvent::Key(k) if k.code == KeyCode::Char('+') => Action::Increment,
            TerminalEvent::Key(k) if k.code == KeyCode::Char('-') => Action::Decrement,
            TerminalEvent::Key(k) if k.code == KeyCode::Tab => match app.active_ui {
                ActiveUI::LEFT => Action::SetActive(ActiveUI::RIGHT),
                ActiveUI::RIGHT => Action::SetActive(ActiveUI::LEFT),
            },
            _ => match app.active_ui {
                ActiveUI::LEFT => self.left_session_list.handle_inner_event(event),
                ActiveUI::RIGHT => self.right_space.handle_inner_event(event),
            },
        }
    }

    pub fn draw(&mut self, app: &mut App, tio: &mut Tio) {
        // TODO: Error handling
        let area = tio.canvas.size().unwrap();
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
            .split(area);
        tio.canvas
            .draw(|f| {
                self.left_session_list.draw(app, f, chunks[0]);
                self.right_space.draw(app, f, chunks[1])
            })
            .unwrap();
    }
}

pub mod right_space {
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
            long_text.push_str("\n");
            let long_text_len = long_text.len();
            messages.push(long_text);

            let mut ret = Self {
                title,
                messages,
                ..Default::default()
            };

            ret.vertical_scroll_state =
                ret.vertical_scroll_state.content_length(ret.messages.len());
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
            self.horizontal_scroll_state =
                self.horizontal_scroll_state.content_length(long_text_len);
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
            frame.render_stateful_widget(
                horizontal_scroll,
                area,
                &mut self.horizontal_scroll_state,
            );
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
}

pub mod left_session_list {
    use crossterm::event::KeyCode;
    use ratatui::prelude::*;
    use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

    use crate::{
        action::Action,
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
                            Action::Bind(self.items[offset].chat_session.clone()),
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
}

use crossterm::event::KeyCode;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState},
};

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::action::{Action, MessagesModelAction};
use crate::app::App;
use crate::models::{ChatSession, RemoteData, SessionRecord};
use crate::tio::TerminalEvent;

use super::{TerminalEventResult, UiEntity, UiId, UiMetaData, UiTag};

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

#[derive(Default)]
pub struct LeftSessionList {
    id: UiId,
    state: ListState,
    items: Vec<Content>,
    meta_data: Rc<UiMetaData>,
    parent: Option<Weak<RefCell<dyn UiEntity>>>,
    boder_style: Style,
    is_highlight: bool,
    // childs: Vec<Rc<RefCell<dyn UiEntity>>>,
}

enum SessionListUI<'a> {
    Already(List<'a>),
    Waiting(List<'a>),
}

impl LeftSessionList {
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

    pub fn with_context_model(&mut self, app: &App) -> &mut Self {
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
        self
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

    fn update_with_context_model(&mut self, app: &App) {
        let mut items = Vec::new();
        match app.sessions_model.get_model_data() {
            RemoteData::Success(data) => {
                data.0
                    .iter()
                    .for_each(|record| items.push(Content::from(record)));
            }
            _ => {}
        }
        self.items = items;
    }

    fn get_ui<'a>(&mut self, app: &App, area: Rect) -> SessionListUI<'a> {
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
                    Line::from(vec![
                        "unread: ".fg(Color::White),
                        content
                            .unread_msg
                            .to_string()
                            .bg(Color::Red)
                            .fg(Color::Black),
                    ]),
                    Line::from("─".repeat(area.width as usize).fg(Color::White)),
                ];
                ListItem::new(lines)
            })
            .collect();

        if items.is_empty() {
            title = "Chats";
            items.push(ListItem::new(Line::from("Waiting for data")));
        }

        SessionListUI::Already(
            List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(title)
                        .border_style(self.boder_style),
                )
                .highlight_style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                ),
        )
    }

    fn next(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.state.selected() {
            Some(i) => {
                if i + 1 >= self.items.len() {
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
        if self.items.is_empty() {
            return;
        }
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

    // fn unselect(&mut self) {
    //     self.state.select(None);
    // }
}

impl UiEntity for LeftSessionList {
    fn make_blueprints(&self, _area: Rect, _ui_mgr: &mut super::ui_manager::UiManager) {
        /* do nothing */
    }

    fn draw(&mut self, app: &App, frame: &mut Frame, area: Rect) {
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

    fn handle_terminal_event(&mut self, event: TerminalEvent) -> super::TerminalEventResult {
        match event {
            TerminalEvent::Key(k) if k.code == KeyCode::Char('j') => {
                self.next();
                TerminalEventResult::Handled(Action::Nop)
            }
            TerminalEvent::Key(k) if k.code == KeyCode::Char('k') => {
                self.prev();
                TerminalEventResult::Handled(Action::Nop)
            }
            TerminalEvent::Key(k) if k.code == KeyCode::Enter => {
                if let Some(offset) = self.state.selected() {
                    // TODO: error handling
                    self.meta_data
                        .set_active_with_tag(&UiTag::MessageViewer)
                        .unwrap();
                    TerminalEventResult::Handled(Action::MessagesModel(
                        MessagesModelAction::SetBind(self.items[offset].chat_session.clone()),
                    ))
                } else {
                    TerminalEventResult::NotHandled(event)
                }
            }
            _ => TerminalEventResult::NotHandled(event),
        }
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

    fn toggle_highlight(&mut self) {
        if self.is_highlight {
            self.is_highlight = false;
            self.boder_style = self.boder_style.fg(Color::Green);
        } else {
            self.is_highlight = true;
            self.boder_style = self.boder_style.fg(Color::White);
        }
    }
}

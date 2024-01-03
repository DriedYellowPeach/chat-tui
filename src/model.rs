/// Model is the module for representing all underlying data for this application
/// Basic types includes: User, Message, Group
///
use tokio::sync::mpsc::UnboundedSender;

use crate::action::Action;

pub enum UserState {
    Online,
    Offline,
    Busy,
    Idle,
    Cusomized(String),
}

pub enum Receiver {
    Individual(UsrID),
    Group(GrpID),
}

#[derive(Debug, Clone)]
pub struct UsrID(String);
#[derive(Debug, Clone)]
pub struct GrpID(String);
#[derive(Debug, Clone)]
pub struct MsgID(String);

pub struct User {
    pub id: UsrID,
    state: UserState,
    uname: String,
    profile: Profile,
}

#[derive(Debug, Default)]
pub struct Profile {
    email: String,
    gender: String,
}

pub struct Group {
    pub id: GrpID,
    // users: Vec<UsrID>,
    owner: UsrID,
    group_name: String,
}

pub struct Message {
    id: MsgID,
    from: UsrID,
    to: Receiver,
    content: String,
    // TODO: time, content
}

pub struct UserPool {
    users: Vec<User>,
}

impl UserPool {
    fn new() -> Self {
        Self { users: Vec::new() }
    }

    fn get_user(&mut self) {
        // Pseudo implementation: init with some fake data
        self.users.push(User {
            id: UsrID(String::from("Alice")),
            state: UserState::Online,
            uname: String::from("Alice"),
            profile: Profile::default(),
        });

        self.users.push(User {
            id: UsrID(String::from("Bob")),
            state: UserState::Online,
            uname: String::from("Bob"),
            profile: Profile::default(),
        });
    }
}

pub struct GroupPool {
    groups: Vec<Group>,
}

pub struct MessagePool {
    messages: Vec<Message>,
}

#[derive(Debug, Clone)]
pub enum ChatSession {
    WithOther(UsrID),
    Group(GrpID),
}

pub struct SessionRecord {
    pub session: ChatSession,
    pub bookmark: MsgID,
    pub unread_msg: usize,
    heat: usize,
}

pub enum RemoteData<T> {
    Success(T),
    Failed,
    Pending,
    Uninitialized,
}

pub struct SessionPool(pub Vec<SessionRecord>);

pub struct SessionsModel {
    is_uninitialized: bool,
    sessions: RemoteData<SessionPool>,
    action_tx: UnboundedSender<Action>,
}

impl SessionsModel {
    pub fn new(action_tx: UnboundedSender<Action>) -> Self {
        Self {
            is_uninitialized: true,
            sessions: RemoteData::Uninitialized,
            action_tx,
        }
    }

    pub fn fetch() -> SessionPool {
        // Pseudo implementation: init with some fake data
        let sessions = vec![
            SessionRecord {
                session: ChatSession::WithOther(UsrID(String::from("Alice"))),
                bookmark: MsgID(String::from("0")),
                unread_msg: 0,
                heat: 0,
            },
            SessionRecord {
                session: ChatSession::WithOther(UsrID(String::from("Bob"))),
                bookmark: MsgID(String::from("0")),
                unread_msg: 1,
                heat: 0,
            },
            SessionRecord {
                session: ChatSession::Group(GrpID(String::from("Nordic Nostalgia"))),
                bookmark: MsgID(String::from("0")),
                unread_msg: 5,
                heat: 0,
            },
        ];

        SessionPool(sessions)
    }

    pub fn load_model_data(&mut self, sp: SessionPool) {
        self.sessions = RemoteData::Success(sp);
    }

    pub fn get_model_data(&mut self) -> RemoteData<&SessionPool> {
        match &self.sessions {
            RemoteData::Uninitialized if self.is_uninitialized => {
                // TODO: error handling
                self.action_tx.send(Action::FetchSessionPool).unwrap();
                self.is_uninitialized = false;
                RemoteData::Pending
            }
            RemoteData::Pending | RemoteData::Failed => {
                // send action to fetch
                RemoteData::Pending
            }
            RemoteData::Success(v) => RemoteData::Success(v),
            _ => RemoteData::Pending,
        }
    }

    pub fn reload_model(&mut self) {
        self.sessions = RemoteData::Pending;
        self.action_tx.send(Action::FetchSessionPool).unwrap();
    }
}

pub struct MessagesModel {
    pub bind: Option<ChatSession>,
    pub messages: RemoteData<Vec<MsgID>>,
    action_tx: UnboundedSender<Action>,
    is_uninitialized: bool,
}

impl MessagesModel {
    pub fn new(action_tx: UnboundedSender<Action>) -> Self {
        Self {
            bind: None,
            messages: RemoteData::Uninitialized,
            action_tx,
            is_uninitialized: true,
        }
    }

    pub fn set_bind(&mut self, session: ChatSession) {
        self.bind = Some(session.clone());
        self.messages = RemoteData::Pending;
        // TODO: Error handling
        self.action_tx.send(Action::FetchMessages(session)).unwrap();
        self.is_uninitialized = false;
    }

    // TODO: pseudo implementation here
    pub fn fetch(_session: ChatSession) -> Vec<MsgID> {
        (1..100).map(|x| MsgID(x.to_string())).collect()
    }

    pub fn load_model_data(&mut self, messages: Vec<MsgID>) {
        self.messages = RemoteData::Success(messages);
    }

    pub fn get_model_data(&mut self) -> RemoteData<&Vec<MsgID>> {
        if let Some(chat_session) = self.bind.clone() {
            match &self.messages {
                RemoteData::Uninitialized if self.is_uninitialized => {
                    // TODO: error handling
                    self.action_tx
                        .send(Action::FetchMessages(chat_session))
                        .unwrap();
                    self.is_uninitialized = false;
                    RemoteData::Pending
                }
                RemoteData::Pending | RemoteData::Failed => {
                    // send action to fetch
                    RemoteData::Pending
                }
                RemoteData::Success(v) => RemoteData::Success(v),
                _ => RemoteData::Pending,
            }
        } else {
            RemoteData::Pending
        }
    }
}

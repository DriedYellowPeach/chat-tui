/// Model is the module for representing all underlying data for this application
/// Basic types includes: User, Message, Group
///
use tokio::sync::mpsc::UnboundedSender;

use crate::action::{Action, MessagesModelAction, SessionsModelAction};

pub mod messages;
pub mod sessions;

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

pub struct SessionPool(pub Vec<SessionRecord>);

pub enum RemoteData<T> {
    Success(T),
    Failed,
    Pending,
    Uninitialized,
}

impl<T> RemoteData<T> {
    pub fn as_ref(&self) -> RemoteData<&T> {
        match *self {
            Self::Success(ref x) => RemoteData::Success(x),
            Self::Failed => RemoteData::Failed,
            Self::Pending => RemoteData::Pending,
            Self::Uninitialized => RemoteData::Uninitialized,
        }
    }
}

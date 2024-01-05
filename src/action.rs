use crate::model::{ChatSession, MsgID, SessionPool};

pub enum SessionsModelAction {
    Reload,
    // fetch and set should be private action
    // cause they are never used by ui components
    Fetch,
    Set(SessionPool),
}

pub enum MessagesModelAction {
    Reload,
    SetBind(ChatSession),
    // below are private actions
    Fetch,
    SetMessages(Vec<MsgID>),
}

pub enum Action {
    SessionsModel(SessionsModelAction),
    MessagesModel(MessagesModelAction),
    Nop,
    Quit,
    MultiAction(Vec<Action>),
}

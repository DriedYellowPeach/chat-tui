use crate::{
    model::{ChatSession, MsgID, SessionPool},
    ui::{UiId, UiTag},
};

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

pub enum UiSharedModelAction {
    IncrementId,
    SetActiveUi(UiId),
    SetUiTag(UiTag, UiId),
}

pub enum Action {
    SessionsModel(SessionsModelAction),
    MessagesModel(MessagesModelAction),
    Nop,
    Quit,
    Increment,
    Decrement,
    MultiAction(Vec<Action>),
}

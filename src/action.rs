use crate::{
    model::{ChatSession, MsgID, SessionPool},
    ui::ActiveUI,
};

pub enum SessionsModelAction {
    Init,
    Reload,
    // fetch and set should be private action
    // cause they are never used by ui components
    Fetch,
    Set(SessionPool),
}

pub enum MessagesModelAction {
    Init,
    Reload,
    SetBind(ChatSession),
    // below are private actions
    Fetch,
    SetMessages(Vec<MsgID>),
}

pub enum Action {
    // SessionsModel
    // FetchSessionPool,
    // ReloadSessionPool,
    // LoadSessionPool(SessionPool),
    SessionsModel(SessionsModelAction),
    // MessagesModel
    MessagesModel(MessagesModelAction),
    // Set Active UI
    SetActive(ActiveUI),
    Nop,
    Quit,
    Increment,
    Decrement,
    MultiAction(Vec<Action>),
    Render,
}

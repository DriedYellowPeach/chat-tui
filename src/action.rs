use crate::{
    model::{ChatSession, MsgID, SessionPool},
    ui::ActiveUI,
};

pub enum Action {
    // SessionsModel
    FetchSessionPool,
    ReloadSessionPool,
    LoadSessionPool(SessionPool),
    // MessagesModel
    FetchMessages(ChatSession),
    Bind(ChatSession),
    LoadMessages(Vec<MsgID>),
    // Set Active UI
    SetActive(ActiveUI),
    Nop,
    Quit,
    Increment,
    Decrement,
    MultiAction(Vec<Action>),
    Render,
}

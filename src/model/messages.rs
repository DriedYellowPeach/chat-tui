use tokio::sync::mpsc::UnboundedSender;

use std::time::Duration;

use crate::action::{Action, MessagesModelAction};

use super::{ChatSession, MsgID, RemoteData, UsrID};

pub struct MessagesModel {
    pub bind: Option<ChatSession>,
    pub messages: RemoteData<Vec<MsgID>>,
    action_tx: UnboundedSender<Action>,
}

impl MessagesModel {
    pub fn new(action_tx: UnboundedSender<Action>) -> Self {
        Self {
            bind: None,
            messages: RemoteData::Uninitialized,
            action_tx,
        }
    }

    // TODO: pseudo implementation here
    pub fn fetch(session: ChatSession) -> Vec<MsgID> {
        match session {
            ChatSession::WithOther(usr) if usr.0 == "SystemBotRaphina" => {
                vec![MsgID("Hello, I'm Raphina. I'm a bot.".to_string())]
            }
            _ => (1..100).map(|x| MsgID(x.to_string())).collect(),
        }
    }

    pub fn get_model_data(&mut self) -> RemoteData<&Vec<MsgID>> {
        self.messages.as_ref()
    }

    pub fn handle_action(&mut self, action: MessagesModelAction) {
        match action {
            MessagesModelAction::Init => self.act_on_init(),
            MessagesModelAction::Fetch => self.act_on_fetch(),
            MessagesModelAction::Reload => self.act_on_reload(),
            MessagesModelAction::SetBind(v) => self.act_on_set_bind(v),
            MessagesModelAction::SetMessages(data) => self.act_on_set_messages(data),
        }
    }

    fn act_on_init(&mut self) {
        self.messages = RemoteData::Pending;
        self.bind = Some(ChatSession::WithOther(UsrID(
            "SystemBotRaphina".to_string(),
        )));
        self.action_tx
            .send(Action::MessagesModel(MessagesModelAction::Fetch))
            .unwrap();
    }

    fn act_on_reload(&mut self) {
        // TODO: maybe reuse the garbage here, not drop vec entirely, keep the space
        // currently I just keep things simple
        self.messages = RemoteData::Pending;
        self.action_tx
            .send(Action::MessagesModel(MessagesModelAction::Fetch))
            .unwrap();
    }

    fn act_on_set_bind(&mut self, session: ChatSession) {
        self.bind = Some(session);
        // set to pending is not always the solution
        // sometimes, there may have cache to use
        self.messages = RemoteData::Pending;
        self.action_tx
            .send(Action::MessagesModel(MessagesModelAction::Fetch))
            .unwrap();
    }

    fn act_on_fetch(&self) {
        // TODO: DEBUG HERE
        assert!(self.bind.is_some());
        if let Some(session) = self.bind.clone() {
            let _tx = self.action_tx.clone();
            tokio::spawn(async move {
                // TODO: fetch will be a async funtion, currently we use sleep to simulate
                tokio::time::sleep(Duration::from_secs(3)).await;
                let data = Self::fetch(session);
                _tx.send(Action::MessagesModel(MessagesModelAction::SetMessages(
                    data,
                )))
                .unwrap();
            });
        }
    }

    fn act_on_set_messages(&mut self, data: Vec<MsgID>) {
        self.messages = RemoteData::Success(data);
    }
}

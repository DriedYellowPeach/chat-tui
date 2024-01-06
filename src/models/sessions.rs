use tokio::sync::mpsc::UnboundedSender;

use std::time::Duration;

use crate::action::{Action, SessionsModelAction};

use super::{ChatSession, GrpID, MsgID, RemoteData, SessionPool, SessionRecord, UsrID};

pub struct SessionsModel {
    sessions: RemoteData<SessionPool>,
    action_tx: UnboundedSender<Action>,
}

impl SessionsModel {
    pub fn new(action_tx: UnboundedSender<Action>) -> Self {
        Self {
            sessions: RemoteData::Uninitialized,
            action_tx,
        }
    }

    pub fn get_model_data(&self) -> RemoteData<&SessionPool> {
        match &self.sessions {
            RemoteData::Uninitialized => {
                self.action_tx
                    .send(Action::SessionsModel(SessionsModelAction::Fetch))
                    .unwrap();
                RemoteData::Pending
            }
            _ => self.sessions.as_ref(),
        }
    }

    pub fn handle_action(&mut self, action: SessionsModelAction) {
        match action {
            // SessionsModelAction::Init => {
            //     self.act_on_init();
            // }
            SessionsModelAction::Reload => {
                self.act_on_reload();
            }
            SessionsModelAction::Fetch => {
                self.act_on_fetch();
            }
            SessionsModelAction::Set(v) => {
                self.act_on_set(v);
            }
        }
    }

    fn fetch() -> SessionPool {
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

    // TODO: should the data set to empty pending or use the cached data to display?
    // Better statemachine on RemoteData
    fn act_on_reload(&mut self) {
        // TODO: maybe reuse the garbage here, not drop vec entirely, keep the space
        // currently I just keep things simple
        self.sessions = RemoteData::Pending;
        self.action_tx
            .send(Action::SessionsModel(SessionsModelAction::Fetch))
            .unwrap();
    }

    fn act_on_fetch(&mut self) {
        let _tx = self.action_tx.clone();

        tokio::spawn(async move {
            // TODO: fetch will be a async funtion, currently we use sleep to simulate
            tokio::time::sleep(Duration::from_secs(3)).await;
            let data = SessionsModel::fetch();
            _tx.send(Action::SessionsModel(SessionsModelAction::Set(data)))
                .unwrap();
        });
    }

    fn act_on_set(&mut self, data: SessionPool) {
        self.sessions = RemoteData::Success(data);
    }
}

use color_eyre::eyre::Result;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use std::time::Duration;

use crate::{
    action::Action,
    model::{MessagesModel, SessionsModel},
    tio::Tio,
    ui::{ActiveUI, UITree},
};

pub struct App {
    shoud_quit: bool,
    action_tx: UnboundedSender<Action>,
    action_rx: UnboundedReceiver<Action>,
    should_draw: bool,
    pub sessions_model: SessionsModel,
    pub messages_model: MessagesModel,
    pub active_ui: ActiveUI,
    counter: u64,
    // pub tio: Tio,
}

impl App {
    pub fn new() -> Result<Self> {
        let (action_tx, action_rx) = tokio::sync::mpsc::unbounded_channel();

        Ok(Self {
            sessions_model: SessionsModel::new(action_tx.clone()),
            messages_model: MessagesModel::new(action_tx.clone()),
            shoud_quit: false,
            should_draw: false,
            active_ui: ActiveUI::RIGHT,
            counter: 0,
            action_tx,
            action_rx,
        })
    }

    // this function handle or dispatch all actions
    pub fn handle_action(&mut self, action: Action) {
        match action {
            Action::Render => {
                self.should_draw = true;
            }
            // action to update SessionsModel
            Action::FetchSessionPool => {
                // this seems useless now, but fetch should be a async function
                // so we are not able to update sessions_model in the coroutine
                // the solution is to defer this update
                // async spawn invoke
                let _tx = self.action_tx.clone();
                tokio::spawn(async move {
                    // TODO: fetch will be a async funtion, currently we use sleep to simulate
                    tokio::time::sleep(Duration::from_secs(3)).await;
                    // SessionsModel::fetch().await;
                    let sessions = SessionsModel::fetch();
                    _tx.send(Action::LoadSessionPool(sessions)).unwrap();
                });
                // async spawn end
            }
            Action::LoadSessionPool(sp) => {
                self.sessions_model.load_model_data(sp);
            }
            Action::ReloadSessionPool => {
                self.sessions_model.reload_model();
            }
            // action to update MessagesModel
            Action::FetchMessages(chat_session) => {
                let _tx = self.action_tx.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_secs(3)).await;
                    let messages = MessagesModel::fetch(chat_session);
                    _tx.send(Action::LoadMessages(messages)).unwrap();
                });
            }
            Action::LoadMessages(v) => {
                self.messages_model.load_model_data(v);
            }
            Action::Bind(chat_session) => {
                self.messages_model.set_bind(chat_session);
            }
            Action::SetActive(ui) => {
                self.active_ui = ui;
            }
            Action::MultiAction(actions) => {
                for action in actions {
                    self.handle_action(action);
                }
            }
            Action::Quit => self.shoud_quit = true,
            Action::Increment => self.counter += 1,
            Action::Decrement => self.counter -= 1,
            Action::Nop => {}
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tio = Tio::new(4.0, 60.0)?;

        // handle event first, event should be dispatch to UI skeleton
        tio.enter()?;
        let mut ui_tree = UITree::with_context_model(self);
        loop {
            if let Some(evt) = tio.next_event().await {
                let action = ui_tree.handle_base_event(evt, self);
                self.action_tx.send(action)?;
            }

            while let Ok(action) = self.action_rx.try_recv() {
                // this will update data in app
                self.handle_action(action);
            }

            // draw ui here
            if self.should_draw {
                ui_tree.draw(self, &mut tio);
                self.should_draw = false;
            }

            if self.shoud_quit {
                // self.tio.leave()?;
                break;
            }
        }
        tio.leave()?;
        Ok(())
    }
}

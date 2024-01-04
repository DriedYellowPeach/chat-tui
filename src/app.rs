use color_eyre::eyre::Result;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::{
    action::{Action, MessagesModelAction, SessionsModelAction},
    model::{messages::MessagesModel, sessions::SessionsModel},
    tio::Tio,
    ui::{root_window::RootWindow, ActiveUI},
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
            Action::SessionsModel(act) => {
                self.sessions_model.handle_action(act);
            }
            // action to update MessagesModel
            Action::MessagesModel(act) => {
                self.messages_model.handle_action(act);
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

    fn init_model(&mut self) -> Result<()> {
        self.action_tx
            .send(Action::SessionsModel(SessionsModelAction::Init))?;
        self.action_tx
            .send(Action::MessagesModel(MessagesModelAction::Init))?;

        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tio = Tio::new(4.0, 60.0)?;
        self.init_model()?;

        // handle event first, event should be dispatch to UI skeleton
        tio.enter()?;
        let mut ui_tree = RootWindow::with_context_model(self);
        self.action_tx
            .send(Action::SessionsModel(SessionsModelAction::Init))
            .unwrap();
        self.action_tx
            .send(Action::MessagesModel(MessagesModelAction::Init))
            .unwrap();
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

use color_eyre::eyre::Result;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use std::rc::Rc;

use crate::action::Action;
use crate::models::{messages::MessagesModel, sessions::SessionsModel, state::StateModel};
use crate::tio::Tio;
use crate::ui_entities::{
    blueprints::UiBlueprints, root_window::RootWindow, TerminalEventResult, UiEntity, UiMetaData,
};

pub struct App {
    shoud_quit: bool,
    action_tx: UnboundedSender<Action>,
    action_rx: UnboundedReceiver<Action>,
    pub sessions_model: SessionsModel,
    pub messages_model: MessagesModel,
    pub state_model: StateModel,
}

impl App {
    pub fn new() -> Result<Self> {
        let (action_tx, action_rx) = tokio::sync::mpsc::unbounded_channel();

        Ok(Self {
            sessions_model: SessionsModel::new(action_tx.clone()),
            messages_model: MessagesModel::new(action_tx.clone()),
            state_model: StateModel::new(),
            shoud_quit: false,
            action_tx,
            action_rx,
        })
    }

    // this function handle or dispatch all actions
    pub fn handle_action(&mut self, action: Action) {
        match action {
            // action to update SessionsModel
            Action::SessionsModel(act) => {
                self.sessions_model.handle_action(act);
            }
            // action to update MessagesModel
            Action::MessagesModel(act) => {
                self.messages_model.handle_action(act);
            }
            Action::StateModel(act) => {
                self.state_model.handle_action(act);
            }
            Action::MultiAction(actions) => {
                for action in actions {
                    self.handle_action(action);
                }
            }
            Action::Quit => self.shoud_quit = true,
            Action::Nop => {}
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tio = Tio::new(4.0, 60.0)?;
        tio.enter()?;

        // handle event first, event should be dispatch to UI skeleton
        let meta = Rc::new(UiMetaData::new());
        let mut root_window = RootWindow::default()
            .with_metadata(meta)
            .with_context_model(self);

        loop {
            if let Some(evt) = tio.next_event().await {
                let action = match root_window.handle_terminal_event(evt, self) {
                    TerminalEventResult::Handled(act) => act,
                    TerminalEventResult::NotHandled(_evt) => {
                        // TODO: log
                        Action::Nop
                    }
                };
                self.action_tx.send(action)?;
            }

            while let Ok(action) = self.action_rx.try_recv() {
                // this will update data in app
                self.handle_action(action);
            }

            // based on newest model, update ui here
            root_window.update_with_context_model(self);

            // draw ui here
            if root_window.meta_data.get_should_draw() {
                tio.canvas
                    .draw(|f| {
                        let mut ui_blueprints = UiBlueprints::new();
                        root_window.make_blueprints(f.size(), &mut ui_blueprints, 0);
                        ui_blueprints.draw(self, f);
                    })
                    .unwrap();
                root_window.meta_data.set_should_draw(false);
                root_window.meta_data.increment_draw_counter();
            }

            if self.shoud_quit {
                break;
            }
        }
        tio.leave()?;
        Ok(())
    }
}

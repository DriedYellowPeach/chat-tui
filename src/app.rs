use color_eyre::eyre::Result;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use std::rc::Rc;

use crate::{
    action::Action,
    models::{messages::MessagesModel, sessions::SessionsModel},
    tio::Tio,
    ui::{root_window::RootWindow, UiMetaData, UiTag},
};

pub struct App {
    shoud_quit: bool,
    action_tx: UnboundedSender<Action>,
    action_rx: UnboundedReceiver<Action>,
    pub sessions_model: SessionsModel,
    pub messages_model: MessagesModel,
}

impl App {
    pub fn new() -> Result<Self> {
        let (action_tx, action_rx) = tokio::sync::mpsc::unbounded_channel();

        Ok(Self {
            sessions_model: SessionsModel::new(action_tx.clone()),
            messages_model: MessagesModel::new(action_tx.clone()),
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
        let mut ui_tree = RootWindow::default()
            .with_metadata(meta)
            .with_context_model(self)
            .with_tag(UiTag::InputHint);

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
            if ui_tree.meta_data.get_should_draw() {
                ui_tree.draw(self, &mut tio);
                ui_tree.meta_data.increment_draw_counter();
                ui_tree.meta_data.set_should_draw(false);
            }

            if self.shoud_quit {
                break;
            }
        }
        tio.leave()?;
        Ok(())
    }
}

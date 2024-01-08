use color_eyre::eyre::Result;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use std::cell::RefCell;
use std::rc::Rc;

use crate::action::Action;
use crate::models::{messages::MessagesModel, sessions::SessionsModel};
use crate::tio::Tio;
use crate::ui::UiEntity;
use crate::ui::{root_window::RootWindow, ui_manager::UiManager, UiMetaData, UiTag};

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
        let mut tio = Tio::new(4.0, 30.0)?;
        tio.enter()?;

        // handle event first, event should be dispatch to UI skeleton
        let meta = Rc::new(UiMetaData::new());
        let root_window = RootWindow::new(meta.clone(), self);
        meta.set_entity_tag(
            Rc::downgrade(&(root_window.clone() as Rc<RefCell<dyn UiEntity>>)),
            UiTag::RootWindow,
        );
        let mut ui_mgr = UiManager::new(meta.clone(), self.action_tx.clone());

        loop {
            if let Some(evt) = tio.next_event().await {
                // let action = ui_tree.handle_base_event(evt, self);
                // self.action_tx.send(action)?;
                ui_mgr.handle_event(evt, root_window.clone());
            }

            while let Ok(action) = self.action_rx.try_recv() {
                // this will update data in app
                self.handle_action(action);
            }

            //draw ui here
            if meta.get_should_draw() {
                ui_mgr.draw(self, &mut tio, root_window.clone());
                meta.increment_draw_counter();
                meta.set_should_draw(false);
            }

            if self.shoud_quit {
                break;
            }
        }
        tio.leave()?;
        Ok(())
    }
}

use ratatui::layout::Rect;
use tokio::sync::mpsc::UnboundedSender;

use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::rc::Rc;

use crate::action::Action;
use crate::app::App;
use crate::tio::TerminalEvent;
use crate::tio::Tio;

use super::{root_window::RootWindow, TerminalEventResult, UiEntity, UiMetaData};

struct Blueprint {
    entity: Rc<RefCell<dyn UiEntity>>,
    area: Rect,
    // bigger the priority is, sooner the entity will be drawn
    priority: isize,
}

impl Ord for Blueprint {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}

impl PartialOrd for Blueprint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Blueprint {}

impl PartialEq for Blueprint {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

pub struct UiManager {
    meta_data: Rc<UiMetaData>,
    blueprints: BinaryHeap<Blueprint>,
    action_tx: UnboundedSender<Action>,
}

impl UiManager {
    pub fn new(meta_data: Rc<UiMetaData>, action_tx: UnboundedSender<Action>) -> Self {
        Self {
            action_tx,
            meta_data,
            blueprints: BinaryHeap::new(),
        }
    }

    pub fn add_new_blueprint(
        &mut self,
        entity: Rc<RefCell<dyn UiEntity>>,
        area: Rect,
        priority: isize,
    ) {
        let bp = Blueprint {
            entity,
            area,
            priority,
        };

        self.blueprints.push(bp);
    }

    pub fn draw(&mut self, app: &App, tio: &mut Tio, root_window: Rc<RefCell<RootWindow>>) {
        tio.canvas
            .draw(|f| {
                root_window.borrow().make_blueprints(f.size(), self);
                while let Some(bp) = self.blueprints.pop() {
                    bp.entity.borrow_mut().draw(app, f, bp.area);
                }
            })
            .unwrap();
    }

    pub fn handle_event(&self, event: TerminalEvent, root_window: Rc<RefCell<RootWindow>>) {
        // should always be a active?
        // add log: no entity handle this event
        let Some(active) = self.meta_data.get_active_entity() else {
        //     let ev_res = RefCell::borrow_mut(root_window.deref()).handle_terminal_event(event);
        //     match ev_res {
        //         TerminalEventResult::Handled(act) => self.action_tx.send(act).unwrap(),
        //         TerminalEventResult::NotHandled(_evt) => {}
        //     }
        //     let rw_weak: Rc<RefCell<dyn UiEntity>> = root_window.clone();
        //     self.meta_data.set_active_entity(Rc::downgrade(&rw_weak));
            return;
        };

        // if upgrade returns None, this suggest that bug in code
        // if one entity is going to be deleted, it should hand over active to it's parent
        let mut active_ptr = active.upgrade().unwrap().clone();
        let mut evt = root_window.borrow().proxy_event(event);
        loop {
            match RefCell::borrow_mut(&active_ptr).handle_terminal_event(evt) {
                TerminalEventResult::NotHandled(e) => evt = e,
                TerminalEventResult::Handled(act) => {
                    self.action_tx.send(act).unwrap();
                    break;
                }
            }
            if RefCell::borrow(&active_ptr).get_parent().is_none() {
                break;
            }
            let parent = RefCell::borrow(&active_ptr)
                .get_parent()
                .unwrap()
                .upgrade()
                .unwrap()
                .clone();

            active_ptr = parent;
        }
    }
}

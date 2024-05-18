use crossterm::event::KeyCode;
use ratatui::layout::Layout;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders};

use std::cell::RefCell;
use std::rc::Rc;

use crate::action::{Action, MessagesModelAction, StateModelAction};
use crate::app::App;
use crate::models::{state::StateModel, RemoteData};
use crate::tio::TerminalEvent;

use super::chat_item::ChatItem;
use super::{TerminalEventResult, UiEntity, UiId, UiMetaData, UiTag};

#[derive(Default)]
struct InternalState {
    offset: usize,
    selected: Option<usize>,
}

impl InternalState {
    fn select(&mut self, index: Option<usize>) {
        self.selected = index;
        if index.is_none() {
            self.offset = 0;
        }
    }

    pub fn selected(&self) -> Option<usize> {
        self.selected
    }
}

#[derive(Default)]
pub struct LeftSessionList {
    id: UiId,
    tag: Option<UiTag>,
    internal: RefCell<InternalState>,
    meta_data: Rc<UiMetaData>,
    items: Vec<ChatItem>,
}

impl LeftSessionList {
    pub fn with_metadata(self, meta: Rc<UiMetaData>) -> Self {
        let mut ret = self;
        ret.id = meta.next_id();
        ret.meta_data = meta;
        ret
    }

    pub fn with_context_model(self, app: &App) -> Self {
        let mut ret = self;
        let mut items = Vec::new();
        match app.sessions_model.get_model_data() {
            RemoteData::Success(data) => {
                data.0
                    .iter()
                    .for_each(|record| items.push(ChatItem::new(record.session.clone())));
                assert_ne!(items.len(), 0);
            }
            _ => {
                assert_eq!(items.len(), 0);
            }
        }

        ret.items = items;

        ret
    }

    pub fn with_tag(self, tag: UiTag) -> Self {
        let mut ret = self;
        ret.tag = Some(tag);
        ret.meta_data.set_tag(tag, ret.id);
        ret
    }

    // TODO: setup highligth for chat item.
    pub fn update_with_context_model(&mut self, app: &App) {
        let mut items = Vec::new();
        match app.sessions_model.get_model_data() {
            RemoteData::Success(data) => {
                data.0
                    .iter()
                    .for_each(|record| items.push(ChatItem::new(record.session.clone())));
                assert_ne!(items.len(), 0);
            }
            _ => {
                assert_eq!(items.len(), 0);
            }
        }

        if let Some(select_idx) = self.internal.borrow().selected() {
            if select_idx < items.len() {
                items[select_idx].set_highlight();
            }
        }

        self.items = items;
    }

    fn next(&mut self) {
        let mut internal = self.internal.borrow_mut();
        let i = match internal.selected() {
            Some(i) => {
                if i + 1 >= self.items.len() {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };

        internal.select(Some(i));
    }

    fn prev(&mut self) {
        let mut internal = self.internal.borrow_mut();
        if self.items.is_empty() {
            // no need to go prev
            return;
        }

        let i = match internal.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };

        internal.select(Some(i));
    }

    // fn unselect(&mut self) {
    //     self.state.borrow_mut().select(None);
    // }
}

impl UiEntity for LeftSessionList {
    fn draw(&self, app: &App, frame: &mut Frame, area: Rect) {
        let bdr_stl = match app.state_model {
            StateModel::Chats => Style::new().fg(Color::Green),
            _ => Style::default(),
        };
        frame.render_widget(
            Block::default()
                .borders(Borders::ALL)
                .title("Chats")
                .border_style(bdr_stl),
            area,
        );
    }

    fn make_blueprints<'a, 'b>(
        &'a self,
        area: Rect,
        ui_mgr: &mut super::blueprints::UiBlueprints<'b>,
        layer: isize,
    ) where
        'a: 'b,
    {
        let up_layer = layer + 1;
        let inner_area = area.inner(&Margin {
            vertical: 1,
            horizontal: 1,
        });

        let height = inner_area.height;
        // TODO, how to use the left_over
        let (num, _left_over) = (height / 4, height % 4);
        let num = std::cmp::min(num as usize, self.items.len());
        let mut constraints = vec![Constraint::Length(4); num];
        constraints.push(Constraint::Min(4));
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(inner_area);
        // TODO: caculate the offset
        for (idx, item) in self.items.iter().enumerate() {
            ui_mgr.add_new_blueprint(item, layout[idx], up_layer);
        }
    }

    fn handle_terminal_event(&mut self, event: TerminalEvent, _app: &App) -> TerminalEventResult {
        match event {
            TerminalEvent::Key(k) if k.code == KeyCode::Char('j') => {
                self.next();
                TerminalEventResult::Handled(Action::Nop)
            }
            TerminalEvent::Key(k) if k.code == KeyCode::Char('k') => {
                self.prev();
                TerminalEventResult::Handled(Action::Nop)
            }
            TerminalEvent::Key(k) if k.code == KeyCode::Enter => {
                let internal = self.internal.borrow();
                if let Some(offset) = internal.selected() {
                    TerminalEventResult::Handled(Action::MultiAction(vec![
                        Action::StateModel(StateModelAction::SetActive(StateModel::Messages)),
                        Action::MessagesModel(MessagesModelAction::SetBind(
                            self.items[offset].id.clone(),
                        )),
                    ]))
                } else {
                    TerminalEventResult::Handled(Action::Nop)
                }
            }
            _ => TerminalEventResult::NotHandled(event),
        }
    }
}

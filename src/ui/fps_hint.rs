use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::time::Instant;

use crate::app::App;
use crate::tio::TerminalEvent;

use super::{TerminalEventResult, UiEntity, UiId, UiMetaData};

pub struct FpsHint {
    id: UiId,
    tick_rate: f64,
    meta_data: Rc<UiMetaData>,
    last_draw_counter: u64,
    last_tick: Instant,
    last_fps: f64,
    parent: Option<Weak<RefCell<dyn UiEntity>>>,
}

impl Default for FpsHint {
    fn default() -> Self {
        Self {
            id: 0,
            meta_data: Rc::new(UiMetaData::new()),
            tick_rate: 10.0,
            last_draw_counter: 0,
            last_fps: 0.0,
            last_tick: Instant::now(),
            parent: None,
        }
    }
}

impl FpsHint {
    pub fn new(meta: Rc<UiMetaData>) -> Rc<RefCell<Self>> {
        let id = meta.next_id();
        let mut ret = Self {
            id,
            ..Default::default()
        };
        ret.meta_data = meta;
        let ret = Rc::new(RefCell::new(ret));
        let _parent = Rc::downgrade(&ret);

        ret
    }

    pub fn with_parent(&mut self, parent: Weak<RefCell<dyn UiEntity>>) -> &mut Self {
        self.parent = Some(parent);
        self
    }

    pub fn contains_active_entity(&self) -> bool {
        let active = self
            .meta_data
            .get_active_entity()
            .unwrap()
            .upgrade()
            .unwrap();

        let id = active.borrow().get_id();
        self.id == id
    }

    fn get_ui<'a>(&mut self, fps: f64) -> Paragraph<'a> {
        let fps_hint = Paragraph::new(format!("{:.2} fps", fps))
            .block(Block::default().title("fps hint").borders(Borders::ALL));
        fps_hint
    }

    fn count_fps(&mut self) -> f64 {
        if self.last_tick.elapsed().as_secs_f64() < 1.0 / self.tick_rate {
            return self.last_fps;
        }

        let current_draw_counter = self.meta_data.draw_counter.get();
        // this suggest that overflow has happend
        let count = if current_draw_counter < self.last_draw_counter {
            u64::MAX - self.last_draw_counter + current_draw_counter
        } else {
            current_draw_counter - self.last_draw_counter
        };

        let fps = count as f64 / self.last_tick.elapsed().as_secs_f64();
        self.last_draw_counter = self.meta_data.draw_counter.get();
        self.last_tick = Instant::now();
        self.last_fps = fps;

        fps
    }
}

impl UiEntity for FpsHint {
    fn get_parent(&self) -> Option<Weak<RefCell<dyn UiEntity>>> {
        match self.parent {
            Some(ref p) => Some(p.clone()),
            None => None,
        }
    }

    fn handle_terminal_event(&mut self, event: TerminalEvent) -> TerminalEventResult {
        TerminalEventResult::NotHandled(event)
    }

    fn draw(&mut self, _app: &App, frame: &mut Frame, area: Rect) {
        let (x, y) = (area.right(), area.top());
        let corner = Rect::new(x - 13, y, 12, 3);
        frame.render_widget(Clear, corner);
        let fps = self.count_fps();
        frame.render_widget(self.get_ui(fps), corner);
    }

    fn make_blueprints(&self, _area: Rect, _ui_mgr: &mut super::ui_manager::UiManager) {
        /* do nothing */
    }
    fn get_id(&self) -> UiId {
        self.id
    }
}

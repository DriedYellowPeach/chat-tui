use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use std::cell::RefCell;
use std::{rc::Rc, time::Instant};

use crate::app::App;

use super::{UiEntity, UiId, UiMetaData, UiTag};

struct InternalState {
    last_draw_counter: u64,
    last_tick: Instant,
    last_fps: f64,
}

pub struct FpsHint {
    id: UiId,
    tick_rate: f64,
    meta_data: Rc<UiMetaData>,
    tag: Option<UiTag>,
    internal_state: RefCell<InternalState>,
}

impl Default for FpsHint {
    fn default() -> Self {
        Self {
            id: 0,
            meta_data: Rc::new(UiMetaData::new()),
            tick_rate: 10.0,
            tag: None,
            internal_state: RefCell::new(InternalState {
                last_draw_counter: 0,
                last_fps: 0.0,
                last_tick: Instant::now(),
            }),
        }
    }
}

impl FpsHint {
    pub fn with_metadata(self, meta: Rc<UiMetaData>) -> Self {
        let mut ret = self;
        ret.id = meta.next_id();
        ret.meta_data = meta;
        ret
    }

    pub fn with_tag(self, tag: UiTag) -> Self {
        let mut ret = self;
        ret.tag = Some(tag);
        ret.meta_data.set_tag(tag, ret.id);
        ret
    }

    fn get_ui<'a>(&self, fps: f64) -> Paragraph<'a> {
        let fps_hint = Paragraph::new(format!("{:.2} fps", fps))
            .block(Block::default().title("fps hint").borders(Borders::ALL));
        fps_hint
    }

    fn count_fps(&self) -> f64 {
        if self
            .internal_state
            .borrow()
            .last_tick
            .elapsed()
            .as_secs_f64()
            < 1.0 / self.tick_rate
        {
            return self.internal_state.borrow().last_fps;
        }

        let mut internal = self.internal_state.borrow_mut();

        let current_draw_counter = self.meta_data.draw_counter.get();
        // this suggest that overflow has happend
        let count = if current_draw_counter < internal.last_draw_counter {
            u64::MAX - internal.last_draw_counter + current_draw_counter
        } else {
            current_draw_counter - internal.last_draw_counter
        };

        let fps = count as f64 / internal.last_tick.elapsed().as_secs_f64();
        internal.last_draw_counter = self.meta_data.draw_counter.get();
        internal.last_tick = Instant::now();
        internal.last_fps = fps;

        fps
    }

    // pub fn draw(&self, _app: &App, frame: &mut Frame<'_>, area: Rect) {
    //     let (x, y) = (area.right(), area.top());
    //     let corner = Rect::new(x - 13, y, 12, 3);
    //     frame.render_widget(Clear, corner);
    //     let fps = self.count_fps();
    //     frame.render_widget(self.get_ui(fps), corner);
    // }
}

impl UiEntity for FpsHint {
    fn draw(&self, _app: &App, frame: &mut Frame, area: Rect) {
        let (x, y) = (area.right(), area.top());
        let corner = Rect::new(x - 13, y, 12, 3);
        frame.render_widget(Clear, corner);
        let fps = self.count_fps();
        frame.render_widget(self.get_ui(fps), corner);
    }
}

use std::{rc::Rc, time::Instant};

use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::app::App;

use super::{UiId, UiMetaData, UiTag};

pub struct FpsHint {
    id: UiId,
    tick_rate: f64,
    meta_data: Rc<UiMetaData>,
    tag: Option<UiTag>,
    last_draw_counter: u64,
    last_tick: Instant,
    last_fps: f64,
}

impl Default for FpsHint {
    fn default() -> Self {
        Self {
            id: 0,
            meta_data: Rc::new(UiMetaData::new()),
            tick_rate: 10.0,
            tag: None,
            last_draw_counter: 0,
            last_fps: 0.0,
            last_tick: Instant::now(),
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

    pub fn draw(&mut self, _app: &App, frame: &mut Frame<'_>, area: Rect) {
        let (x, y) = (area.right(), area.top());
        let corner = Rect::new(x - 13, y, 12, 3);
        frame.render_widget(Clear, corner);
        let fps = self.count_fps();
        frame.render_widget(self.get_ui(fps), corner);
    }
}

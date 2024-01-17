use ratatui::layout::Rect;
use ratatui::Frame;

use std::collections::BinaryHeap;

use crate::app::App;

use super::UiEntity;

struct Blueprint<'a> {
    entity: &'a dyn UiEntity,
    area: Rect,
    layer: isize,
}

impl<'a> Ord for Blueprint<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_layer_priority = -self.layer;
        let other_layer_priority = -other.layer;
        self_layer_priority.cmp(&other_layer_priority)
    }
}

impl<'a> PartialOrd for Blueprint<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Eq for Blueprint<'a> {}

impl<'a> PartialEq for Blueprint<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.layer == other.layer
    }
}

pub struct UiBlueprints<'a> {
    blueprints: BinaryHeap<Blueprint<'a>>,
}

impl<'a> UiBlueprints<'a> {
    pub fn new() -> Self {
        Self {
            blueprints: BinaryHeap::new(),
        }
    }

    pub fn add_new_blueprint<'ent: 'a, T: UiEntity>(
        &mut self,
        entity: &'ent T,
        area: Rect,
        layer: isize,
    ) {
        let bp = Blueprint {
            entity: entity as &dyn UiEntity,
            area,
            layer,
        };

        self.blueprints.push(bp);
    }

    pub fn draw(&mut self, app: &App, f: &mut Frame) {
        // root_window.make_blueprints(f.size(), self);
        while let Some(bp) = self.blueprints.pop() {
            bp.entity.draw(app, f, bp.area);
        }
    }
}

impl<'a> Default for UiBlueprints<'a> {
    fn default() -> Self {
        Self::new()
    }
}

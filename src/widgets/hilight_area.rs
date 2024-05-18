use ratatui::prelude::*;
use ratatui::style::Style;
use ratatui::widgets::Widget;

#[derive(Default)]
pub struct HighlightArea {
    highlight_style: Style,
}

impl HighlightArea {
    pub fn with_style(self, style: Style) -> Self {
        let mut ret = self;
        ret.highlight_style = style;
        ret
    }
}

impl Widget for HighlightArea {
    fn render(self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, self.highlight_style);
    }
}

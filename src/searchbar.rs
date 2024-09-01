use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, Paragraph},
    Frame,
};

use crate::types::GuiState;

pub(crate) fn draw(frame: &mut Frame, r: Rect, state: &GuiState) {
    let input = Paragraph::new("search")
        .style(Style::default())
        .block(Block::bordered());
    frame.render_widget(input, r);
}

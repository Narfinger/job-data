use std::collections::HashMap;

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Block,
    Frame,
};
use yansi::Paint;

use crate::types::{GuiState, Status};

fn single_val<'a>(st: Status, val: usize, total: usize, color: Color) -> Span<'a> {
    Span::styled(
        format!(
            "{} {}/{} ({:.1}%) | ",
            st,
            val,
            total,
            val as f64 / total as f64
        ),
        Style::default().fg(color),
    )
}

fn stats<'a>(state: &'a GuiState) -> Line<'a> {
    let total = state.rdr.iter().count();
    let todos = state
        .rdr
        .iter()
        .filter(|r| r.status == Status::Todo)
        .count();
    let pending = state
        .rdr
        .iter()
        .filter(|r| r.status == Status::Pending)
        .count();
    let rejected = state
        .rdr
        .iter()
        .filter(|r| r.status == Status::Rejected)
        .count();
    let declined = state
        .rdr
        .iter()
        .filter(|r| r.status == Status::Declined)
        .count();

    let spans = vec![
        single_val(Status::Todo, todos, total, Color::Red),
        single_val(Status::Pending, pending, total, Color::Yellow),
        single_val(Status::Rejected, rejected, total, Color::Green),
        single_val(Status::Declined, declined, total, Color::Green),
    ];
    Line::from(spans)
}

pub(crate) fn draw(frame: &mut Frame, r: Rect, state: &GuiState) {
    let widget = stats(state);
    frame.render_widget(widget, r);
}

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    Frame,
};
use time::Date;

use crate::types::{GuiState, Status, DATE_STRING, FORMAT, NOW};

/// Span for a single value in status
fn single_val<'a>(st: Status, val: usize, total: usize, color: Color) -> Span<'a> {
    let percent = (val as f64 / total as f64) * 100_f64;
    Span::styled(
        format!("{}: {}/{} ({:.1}%) | ", st, val, total, percent),
        Style::default().fg(color),
    )
}

/// Returns a line that gives all the stats
fn stats<'a>(state: &'a GuiState) -> Line<'a> {
    let total = state.rdr.len();
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
    let last = state
        .rdr
        .iter()
        .filter_map(|r| Date::parse(&r.last_action_date, &FORMAT).ok())
        .max()
        .unwrap()
        .format(&FORMAT)
        .unwrap();

    let spans = vec![
        single_val(Status::Todo, todos, total, Color::Red),
        single_val(Status::Pending, pending, total, Color::Yellow),
        single_val(Status::Rejected, rejected, total, Color::Green),
        single_val(Status::Declined, declined, total, Color::Green),
        Span::styled(format!("Overall: {}", total), Style::default()),
        Span::styled(format!(" | Last Edit: {}", last), Style::default()),
        Span::styled(format!(" | Today: {}", *DATE_STRING), Style::default()),
    ];
    Line::from(spans)
}

/// Draw the stats frame
pub(crate) fn draw(frame: &mut Frame, r: Rect, state: &GuiState) {
    let widget = stats(state);
    frame.render_widget(widget, r);
}

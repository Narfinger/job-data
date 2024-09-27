use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    Frame,
};

use crate::types::{GuiState, Status, DATE_STRING, FORMAT};

/// Span for a single value in status
fn single_val<'a>(st: &str, val: usize, total: usize, color: Color) -> Span<'a> {
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
    let pending_iter = state.rdr.iter().filter(|r| r.status == Status::Pending);
    let pending = pending_iter.clone().count();
    let pending_past = pending_iter.filter(|r| r.is_old()).count();
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
        .map(|r| r.get_date())
        .max()
        .unwrap().format(&FORMAT).unwrap();

    let spans = vec![
        single_val("Todo", todos, total, Color::Red),
        single_val("Pnd", pending - pending_past, total, Color::Yellow),
        single_val("Pnd+", pending, total, Color::Yellow),
        single_val("Rej", rejected, total, Color::Green),
        single_val("Decl", declined, total, Color::Green),
        Span::styled(format!("#: {}", total), Style::default()),
        Span::styled(format!(" | Edit: {}", last), Style::default()),
        Span::styled(format!(" | Today: {}", *DATE_STRING), Style::default()),
    ];
    Line::from(spans)
}

/// Draw the stats frame
pub(crate) fn draw(frame: &mut Frame, r: Rect, state: &GuiState) {
    let widget = stats(state);
    frame.render_widget(widget, r);
}

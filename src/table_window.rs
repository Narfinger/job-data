use ratatui::{
    crossterm::event::{self, KeyCode},
    prelude::*,
    widgets::{Row, Table},
};
use std::{collections::HashSet, ops::ControlFlow};

use crate::types::{GuiState, GuiView, Record, Status};

fn draw_record(index: usize, r: &Record) -> Row<'_> {
    let color = match r.status {
        Status::Todo => Color::Red,
        Status::Pending => {
            if r.is_old() {
                Color::DarkGray
            } else {
                Color::Yellow
            }
        }
        Status::Rejected => Color::Green,
        Status::Declined => Color::Green,
    };
    Row::new(vec![
        index.to_string(),
        r.status.to_string(),
        r.last_action_date.to_owned(),
        r.name.to_owned(),
        r.subname.to_owned(),
        r.stage.to_owned(),
    ])
    .style(Style::default().fg(color))
}

/// the filter function of which ones to show
fn gui_filter(
    index: &usize,
    r: &Record,
    view: &GuiView,
    changed_this_execution: &HashSet<usize>,
) -> bool {
    r.status == Status::Todo
        || changed_this_execution.contains(index)
        || match view {
            GuiView::Normal => r.status == Status::Pending && !r.is_old(),
            GuiView::Old => r.status == Status::Todo || r.status == Status::Pending,
            GuiView::All => true,
        }
}

pub(crate) fn draw(frame: &mut Frame, rdr: &mut [Record], state: &mut GuiState) {
    let rows = rdr
        .iter()
        .rev()
        .enumerate()
        .filter(|(index, r)| gui_filter(index, r, &state.view, &state.changed_this_exection))
        .map(|(index, r)| draw_record(index, r));

    // Columns widths are constrained in the same way as Layout...
    let widths = [
        Constraint::Length(5),
        Constraint::Length(20),
        Constraint::Length(30),
        Constraint::Length(30),
        Constraint::Length(30),
        Constraint::Length(20),
    ];
    let table = Table::new(rows, widths)
        .column_spacing(1)
        .header(
            Row::new(vec!["#", "Status", "LastDate", "Name", "Subname", "Info"])
                .style(Style::new().bold()),
        )
        .highlight_style(Style::new().reversed())
        .highlight_symbol(">>");

    frame.render_stateful_widget(table, frame.area(), &mut state.table_state);
}

pub(crate) fn handle_input(
    key: event::KeyEvent,
    state: &mut GuiState,
    rdr: &mut [Record],
) -> ControlFlow<()> {
    match key.code {
        KeyCode::Esc => {
            return ControlFlow::Break(());
        }
        KeyCode::Char('q') => {
            return ControlFlow::Break(());
        }
        KeyCode::Up => {
            state.table_state.select_previous();
        }
        KeyCode::Down => {
            state.table_state.select_next();
        }
        KeyCode::PageUp => {
            state.table_state.select_first();
        }
        KeyCode::PageDown => {
            state.table_state.select_last();
        }
        KeyCode::Enter => {
            if let Some(index) = state.table_state.selected() {
                if let Some(record) = rdr.get_mut(rdr.len() - 1 - index) {
                    record.next_stage();
                    state.changed_this_exection.insert(index);
                }
            }
        }
        KeyCode::Char('a') => {
            state.view = state.view.next();
        }
        KeyCode::Char('s') => {
            let txt = state
                .table_state
                .selected()
                .and_then(|i| rdr.get(rdr.len() - 1 - i))
                .map(|r: &Record| r.stage.clone());
            state.stage_text = Some(txt.clone().unwrap());
        }
        _ => {}
    }
    ControlFlow::Continue(())
}

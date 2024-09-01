use ratatui::{
    crossterm::event::{self, KeyCode},
    prelude::*,
    widgets::{Block, Row, Table},
};
use std::ops::ControlFlow;

use crate::types::{GuiState, Record, Save, Status, WindowFocus};

/// draw a single record
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

/// draw the main table
pub(crate) fn draw(frame: &mut Frame, r: Rect, state: &mut GuiState) {
    let rows = state
        .rdr
        .iter()
        .enumerate()
        .filter(|(index, r)| state.filter(index, r))
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
        .highlight_symbol(">>")
        .block(Block::bordered());

    frame.render_stateful_widget(table, r, &mut state.table_state);
}

/// handle input for the table
pub(crate) fn handle_input(key: event::KeyEvent, state: &mut GuiState) -> ControlFlow<Save> {
    match key.code {
        KeyCode::Esc => {
            return ControlFlow::Break(Save::DoNotSave);
        }
        KeyCode::Char('q') => {
            return ControlFlow::Break(Save::Save);
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
            let real_index = state.get_real_index();
            if let Some(record) = state.rdr.get_mut(real_index) {
                record.next_stage();
                state.changed_this_exection.insert(real_index);
            }
        }
        KeyCode::Char('v') => {
            state.view = state.view.next();
        }
        KeyCode::Char('s') => {
            let real_index = state.get_real_index();
            // yes, the state is on the table index not the real index
            state.changed_this_exection.insert(real_index);
            let txt = state.rdr.get(real_index).unwrap().stage.clone();
            state.focus = WindowFocus::StageEdit(txt, real_index);
        }
        KeyCode::Char('?') => {
            state.focus = WindowFocus::Help;
        }
        KeyCode::Char('/') => {
            state.focus = WindowFocus::Search;
        }
        _ => {}
    }
    ControlFlow::Continue(())
}

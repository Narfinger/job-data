use ratatui::{
    crossterm::event::{self, KeyCode},
    layout::{Constraint, Rect},
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph},
    Frame,
};

use crate::types::{center, GuiState, WindowFocus};

/// draw the info frame
pub(crate) fn draw(frame: &mut Frame, _: Rect, state: &GuiState) {
    let area = center(
        frame.area(),
        Constraint::Percentage(60),
        Constraint::Percentage(50), // top and bottom border + content
    );
    let index = state.get_real_index();
    let record = state.rdr.get(index).unwrap();

    let text = Paragraph::new(vec![
        Line::from(vec![Span::from("Name: "), Span::from(record.name.clone())]),
        Line::from(vec![
            Span::from("Subname: "),
            Span::from(record.subname.clone()),
        ]),
        Line::from(vec![
            Span::from("Date: "),
            Span::from(record.last_action_date.clone()),
        ]),
        Line::from(vec![
            Span::from("Stage: "),
            Span::from(record.stage.clone()),
        ]),
        Line::from(vec![
            Span::from("AddInfo: "),
            Span::from(record.additional_info.clone()),
        ]),
        Line::from(vec![
            Span::from("Status: "),
            Span::from(record.status.to_string()),
        ]),
        Line::from(vec![
            Span::from("Place: "),
            Span::from(record.place.clone()),
        ]),
    ])
    .block(Block::bordered().title("Info"));
    frame.render_widget(Clear, area);
    frame.render_widget(text, area);
}

pub(crate) fn handle_input(key: event::KeyEvent, state: &mut GuiState) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            state.focus = WindowFocus::Table;
        }
        _ => {}
    }
}

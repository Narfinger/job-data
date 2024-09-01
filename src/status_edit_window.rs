use ratatui::{
    crossterm::event::{self, KeyCode},
    layout::{Constraint, Position, Rect},
    widgets::{Block, Clear, Paragraph},
    Frame,
};

use crate::types::{center, WindowFocus, GuiState};

/// draw the status edit frame
pub(crate) fn draw(frame: &mut Frame, _: Rect, state: &GuiState) {
    if let WindowFocus::StageEdit(ref txt, _) = state.focus {
        let area = center(
            frame.area(),
            Constraint::Percentage(20),
            Constraint::Length(3), // top and bottom border + content
        );
        let text_input =
            Paragraph::new(txt.to_owned()).block(Block::bordered().title("Stage Info"));
        frame.render_widget(Clear, area);
        frame.render_widget(text_input, area);
        frame.set_cursor_position(Position::new(
            area.x + txt.len() as u16 + 1,
            // Move one line down, from the border to the input line
            area.y + 1,
        ))
    }
}

/// handle inputs for status edit frame
pub(crate) fn handle_input(key: event::KeyEvent, state: &mut GuiState) {
    let rdr = &mut state.rdr;
    match key.code {
        KeyCode::Esc => {
            state.focus = WindowFocus::Table;
        }
        KeyCode::Enter => {
            if let WindowFocus::StageEdit(ref txt, real_index) = state.focus {
                rdr.get_mut(real_index).unwrap().set_stage(txt.clone());
            }
            state.focus = WindowFocus::Table;
        }
        KeyCode::Char(char) => {
            if let WindowFocus::StageEdit(ref mut txt, _) = state.focus {
                txt.push(char);
            }
        }
        KeyCode::Backspace => {
            if let WindowFocus::StageEdit(ref mut txt, _) = state.focus {
                txt.pop();
            }
        }
        _ => {}
    };
}

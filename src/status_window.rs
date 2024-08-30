use ratatui::{
    crossterm::event::{self, KeyCode},
    layout::{Constraint, Flex, Layout},
    widgets::{Block, Paragraph},
    Frame,
};

use crate::types::{GuiState, Record, Window};

pub(crate) fn draw(frame: &mut Frame, txt: &str) {
    let layout = Layout::vertical([Constraint::Percentage(50)])
        .flex(Flex::Center)
        .vertical_margin(4)
        .split(frame.area());
    let text_input = Paragraph::new(txt.to_owned()).block(Block::bordered().title("Stage Info"));
    frame.render_widget(text_input, layout[0]);
}

pub(crate) fn handle_input(key: event::KeyEvent, state: &mut GuiState, rdr: &mut [Record]) {
    match key.code {
        KeyCode::Esc => {
            state.window = Window::Table;
        }
        KeyCode::Enter => {
            if let Window::StageWindow(ref txt) = state.window {
                rdr.get_mut(rdr.len() - 1 - state.table_state.selected().unwrap())
                    .unwrap()
                    .set_stage(txt.clone());
            }
        }
        KeyCode::Char(char) => {
            if let Window::StageWindow(ref mut txt) = state.window {
                txt.push(char);
            }
        }
        KeyCode::Backspace => {
            if let Window::StageWindow(ref mut txt) = state.window {
                txt.pop();
            }
        }
        _ => {}
    };
}

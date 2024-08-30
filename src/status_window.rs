use ratatui::{
    crossterm::event::{self, KeyCode},
    layout::{Constraint, Flex, Layout},
    widgets::{Block, Paragraph},
    Frame,
};

use crate::types::{GuiState, Record};

pub(crate) fn draw_text_input(frame: &mut Frame, state: &GuiState) {
    let layout = Layout::vertical([Constraint::Percentage(50)])
        .flex(Flex::Center)
        .vertical_margin(4)
        .split(frame.area());
    let text_input = Paragraph::new(state.stage_text.clone().unwrap())
        .block(Block::bordered().title("Stage Info"));
    frame.render_widget(text_input, layout[0]);
}

pub(crate) fn handle_input(key: event::KeyEvent, state: &mut GuiState, rdr: &mut [Record]) {
    match key.code {
        KeyCode::Esc => {
            state.stage_text.take();
        }
        KeyCode::Enter => {
            let txt = state.stage_text.take();
            rdr.get_mut(rdr.len() - 1 - state.table_state.selected().unwrap())
                .unwrap()
                .set_stage(txt.unwrap());
        }
        KeyCode::Char(char) => {
            state.stage_text.as_mut().unwrap().push(char);
        }
        KeyCode::Backspace => {
            state.stage_text.as_mut().unwrap().pop();
        }
        _ => {}
    };
}

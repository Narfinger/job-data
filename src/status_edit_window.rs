use ratatui::{
    crossterm::event::{self, KeyCode},
    layout::{Constraint, Flex, Layout, Position, Rect},
    widgets::{Block, Clear, Paragraph},
    Frame,
};

use crate::{
    table_window,
    types::{GuiState, Window},
};

fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

pub(crate) fn draw(frame: &mut Frame, r: Rect, state: &GuiState) {
    if let Window::StageEdit(ref txt, _) = state.window {
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

pub(crate) fn handle_input(key: event::KeyEvent, state: &mut GuiState) {
    let rdr = &mut state.rdr;
    match key.code {
        KeyCode::Esc => {
            state.window = Window::Table;
        }
        KeyCode::Enter => {
            if let Window::StageEdit(ref txt, real_index) = state.window {
                rdr.get_mut(real_index).unwrap().set_stage(txt.clone());
            }
            state.window = Window::Table;
        }
        KeyCode::Char(char) => {
            if let Window::StageEdit(ref mut txt, _) = state.window {
                txt.push(char);
            }
        }
        KeyCode::Backspace => {
            if let Window::StageEdit(ref mut txt, _) = state.window {
                txt.pop();
            }
        }
        _ => {}
    };
}

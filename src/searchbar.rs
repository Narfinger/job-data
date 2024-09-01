use ratatui::{
    crossterm::event::{self, KeyCode},
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Paragraph},
    Frame,
};

use crate::{
    table_window,
    types::{GuiState, Window},
};

pub(crate) fn draw(frame: &mut Frame, r: Rect, state: &GuiState) {
    let (style, txt) = if state.window == Window::Search {
        (
            Style::default().fg(Color::Green),
            state.search.clone().unwrap_or_default(),
        )
    } else {
        (Style::default(), "search".to_string())
    };
    let input = Paragraph::new(txt).style(style).block(Block::bordered());
    frame.render_widget(input, r);
}

pub(crate) fn handle_input(key: event::KeyEvent, state: &mut GuiState) {
    match key.code {
        KeyCode::Esc => {
            state.search.take();
            state.window = Window::Table;
        }
        KeyCode::Char(k) => {
            if let Some(ref mut s) = state.search {
                s.push(k);
            } else {
                state.search = Some(k.to_string());
            }
        }
        _ => {
            table_window::handle_input(key, state);
        }
    }
}

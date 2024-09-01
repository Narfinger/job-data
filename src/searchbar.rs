use ratatui::{
    crossterm::event::{self, KeyCode},
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Paragraph},
    Frame,
};

use crate::{
    table_window,
    types::{GuiState, WindowFocus},
};

/// draw the search input field
pub(crate) fn draw(frame: &mut Frame, r: Rect, state: &GuiState) {
    let (style, txt) = if state.focus == WindowFocus::Search {
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

/// handle search input and defer to table if we do not know what to do with it
pub(crate) fn handle_input(key: event::KeyEvent, state: &mut GuiState) {
    match key.code {
        KeyCode::Esc => {
            state.search.take();
            state.focus = WindowFocus::Table;
        }
        KeyCode::Char(k) => {
            if let Some(ref mut s) = state.search {
                s.push(k);
            } else {
                state.search = Some(k.to_string());
            }
        }
        _ => {
            // we still want normal stuff
            table_window::handle_input(key, state);
        }
    }
}

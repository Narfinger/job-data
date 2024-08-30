use ratatui::{
    crossterm::event::{self, KeyCode},
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List},
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

fn styled_text<'a>(key: &'a str, text: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::styled("- ", Style::default()),
        Span::styled(key, Style::default().fg(Color::Green)),
        Span::styled(": ", Style::default()),
        Span::styled(text, Style::default()),
    ])
}

pub(crate) fn draw(frame: &mut Frame, r: Rect, state: &mut GuiState) {
    // first draw the table
    table_window::draw(frame, r, state);

    let area = center(
        frame.area(),
        Constraint::Percentage(30),
        Constraint::Length(30), // top and bottom border + content
    );
    let help = (List::new([
        styled_text("Esc", "to exit without saving"),
        styled_text("q", "to exit wit saving"),
        styled_text("v", "to change the stage visiblity"),
        styled_text("s", "to change stage"),
        styled_text("Enter", "to toggle through status"),
        styled_text("?", "help"),
    ]))
    .block(Block::new().borders(Borders::ALL));
    frame.render_widget(Clear, area);
    frame.render_widget(help, area);
}

pub(crate) fn handle_input(key: event::KeyEvent, state: &mut GuiState) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            state.window = Window::Table;
        }
        _ => {}
    }
}

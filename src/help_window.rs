use ratatui::{
    crossterm::event::{self, KeyCode},
    layout::{Constraint, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List},
    Frame,
};

use crate::types::{center, GuiState, WindowFocus};

/// helper function for the help style text coloring
fn styled_text<'a>(key: &'a str, text: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::styled("- ", Style::default()),
        Span::styled(key, Style::default().fg(Color::Green)),
        Span::styled(": ", Style::default()),
        Span::styled(text, Style::default()),
    ])
}

/// draw the help window
pub(crate) fn draw(frame: &mut Frame, _: Rect, _: &GuiState) {
    let area = center(
        frame.area(),
        Constraint::Percentage(30),
        Constraint::Length(30), // top and bottom border + content
    );
    let help = (List::new([
        styled_text("Esc", "to exit without saving"),
        styled_text("q", "to exit wit saving"),
        styled_text("Enter", "to toggle through status"),
        styled_text("Delete", "delete an entry"),
        styled_text("v", "toggle which entries we see"),
        styled_text("s", "to change stage"),
        styled_text("?", "help"),
        styled_text("/", "search the names"),
        styled_text("a", "add a job"),
        styled_text("i", "information about highlighted job"),
        styled_text("e", "edit the entry"),
    ]))
    .block(Block::new().borders(Borders::ALL));
    frame.render_widget(Clear, area);
    frame.render_widget(help, area);
}

/// help window input handler
pub(crate) fn handle_input(key: event::KeyEvent, state: &mut GuiState) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            state.focus = WindowFocus::Table;
        }
        _ => {}
    }
}

use ratatui::{
    crossterm::event::{self, KeyCode},
    layout::{Constraint, Layout, Position, Rect},
    style::{Color, Style},
    widgets::{Block, Clear, Paragraph},
    Frame,
};
use yansi::Paint;

use crate::types::{center, AddFocusField, GuiState, Record, WindowFocus};

/// draw the add window
pub(crate) fn draw(frame: &mut Frame, _: Rect, state: &GuiState) {
    let area = center(
        frame.area(),
        Constraint::Percentage(30),
        Constraint::Length(30), // top and bottom border + content
    );
    let s = state.add.as_ref().unwrap();

    let highlight_style = Style::default().fg(Color::Green);
    let company =
        Paragraph::new(s.company.clone()).block(Block::bordered().title("Company").style(
            if s.focus == AddFocusField::Company {
                highlight_style
            } else {
                Style::default()
            },
        ));
    let subname = Paragraph::new(s.jobname.clone())
        .block(Block::bordered().title("job name"))
        .style(if s.focus == AddFocusField::JobName {
            highlight_style
        } else {
            Style::default()
        });
    let l =
        Layout::vertical(vec![Constraint::Percentage(50), Constraint::Percentage(50)]).split(area);

    let (x, y) = match s.focus {
        AddFocusField::Company => (l[0].x + 1 + s.company.len() as u16, l[0].y + 1),
        AddFocusField::JobName => (l[1].x + 1 + s.jobname.len() as u16, l[1].y + 1),
    };
    frame.render_widget(Clear, area);
    frame.render_widget(company, l[0]);
    frame.render_widget(subname, l[1]);
    frame.set_cursor_position(Position::new(x, y))
}

/// add window input handler
pub(crate) fn handle_input(key: event::KeyEvent, state: &mut GuiState) {
    match key.code {
        KeyCode::Esc => {
            state.focus = WindowFocus::Table;
        }
        KeyCode::Up | KeyCode::Down | KeyCode::Tab => {
            let res = match state.add.as_ref().unwrap().focus {
                AddFocusField::Company => AddFocusField::JobName,
                AddFocusField::JobName => AddFocusField::Company,
            };
            state.add.as_mut().unwrap().focus = res;
        }
        KeyCode::Char(c) => match state.add.as_ref().unwrap().focus {
            AddFocusField::Company => state.add.as_mut().unwrap().company.push(c),
            AddFocusField::JobName => state.add.as_mut().unwrap().jobname.push(c),
        },
        KeyCode::Enter => {
            let s = state.add.as_ref().unwrap();
            let record = Record::new(s.company.clone(), s.jobname.clone());
            state.rdr.push(record);
            state.add = None;
            state.table_state.select_last();
            state.focus = WindowFocus::Table;
        }
        KeyCode::Backspace => {
            match state.add.as_ref().unwrap().focus {
                AddFocusField::Company => state.add.as_mut().unwrap().company.pop(),
                AddFocusField::JobName => state.add.as_mut().unwrap().jobname.pop(),
            };
        }
        _ => {}
    }
}

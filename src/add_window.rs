use ratatui::{
    crossterm::event::{self, KeyCode},
    layout::{Constraint, Layout, Position, Rect},
    style::{Color, Style},
    widgets::{Block, Clear, Paragraph},
    Frame,
};

use crate::types::{center, AddFocusField, AddStruct, GuiState, Record, WindowFocus};

fn paragraph<'a>(
    s: &AddStruct,
    focus: AddFocusField,
    text: &'a str,
    title: &'a str,
) -> Paragraph<'a> {
    let highlight_style = Style::default().fg(Color::Green);
    Paragraph::new(text).block(Block::bordered().title(title).style(if s.focus == focus {
        highlight_style
    } else {
        Style::default()
    }))
}

/// draw the add window
pub(crate) fn draw(frame: &mut Frame, _: Rect, state: &GuiState) {
    let area = center(
        frame.area(),
        Constraint::Percentage(30),
        Constraint::Length(10), // top and bottom border + content
    );
    let s = state.add.as_ref().unwrap();
    let company = paragraph(s, AddFocusField::Company, &s.company, "Company");
    let subname = paragraph(s, AddFocusField::JobName, &s.jobname, "JobName");
    let place = paragraph(s, AddFocusField::Place, &s.place, "Place");

    let l = Layout::vertical(vec![
        Constraint::Percentage(33),
        Constraint::Percentage(33),
        Constraint::Percentage(33),
    ])
    .split(area);

    let (x, y) = match s.focus {
        AddFocusField::Company => (l[0].x + 1 + s.company.len() as u16, l[0].y + 1),
        AddFocusField::JobName => (l[1].x + 1 + s.jobname.len() as u16, l[1].y + 1),
        AddFocusField::Place => (l[2].x + 1 + s.jobname.len() as u16, l[2].y + 1),
    };
    frame.render_widget(Clear, area);
    frame.render_widget(company, l[0]);
    frame.render_widget(subname, l[1]);
    frame.render_widget(place, l[2]);
    frame.set_cursor_position(Position::new(x, y))
}

/// add window input handler
pub(crate) fn handle_input(key: event::KeyEvent, state: &mut GuiState) {
    match key.code {
        KeyCode::Esc => {
            state.focus = WindowFocus::Table;
        }
        KeyCode::Up | KeyCode::Tab => {
            state.add.as_mut().unwrap().focus = state.add.as_ref().unwrap().focus.prev();
        }
        KeyCode::Down => {
            state.add.as_mut().unwrap().focus = state.add.as_ref().unwrap().focus.next();
        }
        KeyCode::Char(c) => match state.add.as_ref().unwrap().focus {
            AddFocusField::Company => state.add.as_mut().unwrap().company.push(c),
            AddFocusField::JobName => state.add.as_mut().unwrap().jobname.push(c),
            AddFocusField::Place => state.add.as_mut().unwrap().place.push(c),
        },
        KeyCode::Enter => {
            let s = state.add.as_ref().unwrap();
            let record = Record::new(s.company.clone(), s.jobname.clone(), s.place.clone());
            state.rdr.push(record);
            state.add = None;
            state.table_state.select_last();
            state.focus = WindowFocus::Table;
        }
        KeyCode::Backspace => {
            match state.add.as_ref().unwrap().focus {
                AddFocusField::Company => state.add.as_mut().unwrap().company.pop(),
                AddFocusField::JobName => state.add.as_mut().unwrap().jobname.pop(),
                AddFocusField::Place => state.add.as_mut().unwrap().place.pop(),
            };
        }
        _ => {}
    }
}

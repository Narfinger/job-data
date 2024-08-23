use anyhow::Context;
use layout::Rows;
use ratatui::{
    crossterm::{
        event::{self, KeyCode, KeyEventKind},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    prelude::*,
    widgets::{Block, Row, Table, TableState},
};
use std::io::stdout;
use time::{format_description, Date, OffsetDateTime, UtcOffset};

use crate::types::{Record, Status};

fn draw_record(r: &Record) -> Vec<String> {
    vec![
        r.status.to_string(),
        r.last_action_date.to_owned(),
        r.name.to_owned(),
        r.subname.to_owned(),
        r.stage.to_owned(),
    ]
}
fn draw(rdr: &mut [Record]) -> impl StatefulWidget<State = TableState> {
    let rows = rdr
        .iter()
        .filter(|r| r.status == Status::Pending || r.status == Status::Todo)
        .map(|r| Row::new(draw_record(r)));

    // Columns widths are constrained in the same way as Layout...
    let widths = [
        Constraint::Length(10),
        Constraint::Length(20),
        Constraint::Length(30),
        Constraint::Length(30),
        Constraint::Length(20),
    ];
    Table::new(rows, widths)
        .column_spacing(1)
        .header(
            Row::new(vec!["Status", "LastDate", "Name", "Subname", "Info"])
                .style(Style::new().bold()),
        )
        .block(Block::new().title("Table"))
        .highlight_style(Style::new().reversed())
        .highlight_symbol(">>")
}

pub(crate) fn run(rdr: &mut [Record]) -> anyhow::Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    // TODO main loop
    let mut table_state = TableState::default();
    table_state.select_first();
    loop {
        terminal.draw(|frame| {
            let area = frame.area();
            frame.render_stateful_widget(draw(rdr), area, &mut table_state);
        })?;
        // TODO handle events
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Up => {
                            table_state.select_previous();
                        }
                        KeyCode::Down => {
                            table_state.select_next();
                        }
                        KeyCode::Char('i') => {
                            
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

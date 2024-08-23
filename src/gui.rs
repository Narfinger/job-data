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

use crate::types::{Record, Status};

fn draw_record<'a>(index: usize, r: &'a Record) -> Row<'a> {
    let color = match r.status {
        Status::Todo => Color::Green,
        Status::Pending => Color::Yellow,
        Status::Rejected => Color::Red,
        Status::Declined => Color::Red,
    };
    Row::new(vec![
        index.to_string(),
        r.status.to_string(),
        r.last_action_date.to_owned(),
        r.name.to_owned(),
        r.subname.to_owned(),
        r.stage.to_owned(),
    ])
    .style(Style::default().fg(color))
}
fn draw<'a>(rdr: &'a mut [Record]) -> impl StatefulWidget<State = TableState> + 'a {
    let rows = rdr
        .iter()
        .enumerate()
        .map(|(index, r)| draw_record(index, r));

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
                        KeyCode::Enter => {
                            if let Some(record) =
                                table_state.selected().and_then(|i| rdr.get_mut(i))
                            {
                                record.next_stage();
                            }
                        }
                        KeyCode::PageUp => {
                            table_state.select_first();
                        }
                        KeyCode::PageDown => {
                            table_state.select_last();
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

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

use crate::types::{GuiView, Record, Save, Status};

struct GuiState {
    table_state: TableState,
    view: GuiView,
}

fn draw_record(index: usize, r: &Record) -> Row<'_> {
    let color = match r.status {
        Status::Todo => Color::Red,
        Status::Pending => {
            if r.is_old() {
                Color::DarkGray
            } else {
                Color::Yellow
            }
        }
        Status::Rejected => Color::Green,
        Status::Declined => Color::Green,
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

fn gui_filter(r: &Record, view: &GuiView) -> bool {
    r.status == Status::Todo
        || match view {
            GuiView::Normal => r.status == Status::Pending && !r.is_old(),
            GuiView::Old => r.status == Status::Todo || r.status == Status::Pending,
            GuiView::All => true,
        }
}

fn draw<'a>(rdr: &'a mut [Record], view: GuiView) -> impl StatefulWidget<State = TableState> + 'a {
    let rows = rdr
        .iter()
        .rev()
        .filter(|r| gui_filter(r, &view))
        .enumerate()
        .map(|(index, r)| draw_record(index, r));

    // Columns widths are constrained in the same way as Layout...
    let widths = [
        Constraint::Length(5),
        Constraint::Length(20),
        Constraint::Length(30),
        Constraint::Length(30),
        Constraint::Length(30),
        Constraint::Length(20),
    ];
    Table::new(rows, widths)
        .column_spacing(1)
        .header(
            Row::new(vec!["#", "Status", "LastDate", "Name", "Subname", "Info"])
                .style(Style::new().bold()),
        )
        .block(Block::new().title("Table"))
        .highlight_style(Style::new().reversed())
        .highlight_symbol(">>")
}

pub(crate) fn run(rdr: &mut [Record]) -> anyhow::Result<Save> {
    let save_state;
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut state = GuiState {
        table_state: TableState::default().with_selected(Some(0)),
        view: GuiView::Normal,
    };

    loop {
        terminal.draw(|frame| {
            let area = frame.area();
            frame.render_stateful_widget(draw(rdr, state.view), area, &mut state.table_state);
        })?;
        // TODO handle events
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Esc => {
                            save_state = Save::DoNotSave;
                            break;
                        }
                        KeyCode::Char('q') => {
                            save_state = Save::Save;
                            break;
                        }
                        KeyCode::Up => {
                            state.table_state.select_previous();
                        }
                        KeyCode::Down => {
                            state.table_state.select_next();
                        }
                        KeyCode::PageUp => {
                            state.table_state.select_first();
                        }
                        KeyCode::PageDown => {
                            state.table_state.select_last();
                        }
                        KeyCode::Enter => {
                            if let Some(record) = state
                                .table_state
                                .selected()
                                .and_then(|i| rdr.get_mut(rdr.len() - 1 - i))
                            {
                                record.next_stage();
                            }
                        }
                        KeyCode::Char('a') => {
                            state.view = state.view.next();
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(save_state)
}

use layout::Flex;
use ratatui::{
    crossterm::{
        event::{self, KeyCode, KeyEventKind},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    prelude::*,
    widgets::{Block, Paragraph, Row, Table, TableState},
};
use std::{io::stdout, ops::ControlFlow};

use crate::types::{GuiView, Record, Save, Status};

struct GuiState {
    table_state: TableState,
    view: GuiView,
    stage_text: Option<String>,
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

fn draw_text_input(frame: &mut Frame, state: &GuiState) {
    let layout = Layout::vertical([Constraint::Percentage(50)])
        .flex(Flex::Center)
        .vertical_margin(4)
        .split(frame.area());
    let text_input = Paragraph::new(state.stage_text.clone().unwrap())
        .block(Block::bordered().title("Stage Info"));
    frame.render_widget(text_input, layout[0]);
}

fn draw_table(frame: &mut Frame, rdr: &mut [Record], state: &mut GuiState) {
    let rows = rdr
        .iter()
        .rev()
        .filter(|r| gui_filter(r, &state.view))
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
    let table = Table::new(rows, widths)
        .column_spacing(1)
        .header(
            Row::new(vec!["#", "Status", "LastDate", "Name", "Subname", "Info"])
                .style(Style::new().bold()),
        )
        .highlight_style(Style::new().reversed())
        .highlight_symbol(">>");

    frame.render_stateful_widget(table, frame.area(), &mut state.table_state);
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
        stage_text: None,
    };

    loop {
        terminal.draw(|frame| {
            if state.stage_text.is_some() {
                draw_text_input(frame, &state);
            } else {
                draw_table(frame, rdr, &mut state)
            }
        })?;
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if state.stage_text.is_some() {
                        handle_text_input(key, &mut state, rdr);
                    } else if let (save, ControlFlow::Break(_)) =
                        handle_table_input(key, &mut state, rdr)
                    {
                        save_state = save;
                        break;
                    }
                }
            }
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(save_state)
}

fn handle_text_input(key: event::KeyEvent, state: &mut GuiState, rdr: &mut [Record]) {
    match key.code {
        KeyCode::Esc => {
            state.stage_text.take();
        }
        KeyCode::Enter => {
            let txt = state.stage_text.take();
            rdr.get_mut(rdr.len() - 1 - state.table_state.selected().unwrap())
                .unwrap()
                .set_stage(txt.unwrap());
        }
        KeyCode::Char(char) => {
            state.stage_text.as_mut().unwrap().push(char);
        }
        KeyCode::Backspace => {
            state.stage_text.as_mut().unwrap().pop();
        }
        _ => {}
    };
}

fn handle_table_input(
    key: event::KeyEvent,
    state: &mut GuiState,
    rdr: &mut [Record],
) -> (Save, ControlFlow<()>) {
    match key.code {
        KeyCode::Esc => {
            return (Save::Save, ControlFlow::Break(()));
        }
        KeyCode::Char('q') => {
            return (Save::DoNotSave, ControlFlow::Break(()));
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
        KeyCode::Char('s') => {
            let txt = state
                .table_state
                .selected()
                .and_then(|i| rdr.get(rdr.len() - 1 - i))
                .map(|r: &Record| r.stage.clone());
            state.stage_text = Some(txt.unwrap());
        }
        _ => {}
    }
    (Save::Save, ControlFlow::Continue(()))
}

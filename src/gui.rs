use ratatui::{
    crossterm::{
        event::{self, KeyEventKind},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    prelude::*,
    widgets::TableState,
};
use std::{collections::HashSet, io::stdout, ops::ControlFlow};

use crate::{
    status_window, table_window,
    types::{GuiState, GuiView, Record},
};

pub(crate) fn run(rdr: &mut [Record]) -> anyhow::Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut state = GuiState {
        table_state: TableState::default().with_selected(Some(0)),
        view: GuiView::Normal,
        stage_text: None,
        changed_this_exection: HashSet::new(),
    };

    loop {
        terminal.draw(|frame| {
            if state.stage_text.is_some() {
                status_window::draw_text_input(frame, &state);
            } else {
                table_window::draw(frame, rdr, &mut state)
            }
        })?;
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if state.stage_text.is_some() {
                        status_window::handle_input(key, &mut state, rdr);
                    } else if let ControlFlow::Break(_) =
                        table_window::handle_input(key, &mut state, rdr)
                    {
                        break;
                    }
                }
            }
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

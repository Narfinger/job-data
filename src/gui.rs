use ratatui::{
    crossterm::{
        event::{self, KeyEventKind},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    prelude::*,
    widgets::TableState,
};
use std::{collections::HashSet, io::stdout};

use crate::{
    status_window, table_window,
    types::{GuiState, GuiView, Record, Window},
};

pub(crate) fn run(rdr: &mut [Record]) -> anyhow::Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut state = GuiState {
        table_state: TableState::default().with_selected(Some(0)),
        view: GuiView::Normal,
        window: Window::Table,
        changed_this_exection: HashSet::new(),
    };

    loop {
        terminal.draw(|frame| {
            match &state.window {
                Window::Table => table_window::draw(frame, rdr, &mut state),
                Window::StageWindow(txt) => status_window::draw(frame, txt),
            };
        })?;
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match state.window {
                        Window::Table => {
                            if table_window::handle_input(key, &mut state, rdr).is_break() {
                                break;
                            }
                        }
                        Window::StageWindow(_) => {
                            status_window::handle_input(key, &mut state, rdr);
                        }
                    };
                }
            }
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

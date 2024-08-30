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
    help_window, status_window, table_window,
    types::{GuiState, GuiView, Record, Save, Window},
};

pub(crate) fn run(rdr: &mut [Record]) -> anyhow::Result<Save> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut state = GuiState {
        rdr,
        table_state: TableState::default().with_selected(Some(0)),
        view: GuiView::Normal,
        window: Window::Table,
        changed_this_exection: HashSet::new(),
    };

    let save;
    loop {
        terminal.draw(|frame| {
            match &state.window {
                Window::Table => table_window::draw(frame, &mut state),
                Window::StageEdit(_) => status_window::draw(frame, &mut state),
                Window::Help => help_window::draw(frame, &mut state),
            };
        })?;
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match state.window {
                        Window::Table => {
                            if let ControlFlow::Break(s) =
                                table_window::handle_input(key, &mut state)
                            {
                                save = s;
                                break;
                            }
                        }
                        Window::StageEdit(_) => {
                            status_window::handle_input(key, &mut state);
                        }
                        Window::Help => help_window::handle_input(key, &mut state),
                    };
                }
            }
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(save)
}

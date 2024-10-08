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
    add_window, help_window, info_window,
    records::Records,
    searchbar, status_edit_window, summarybar, table_window,
    types::{GuiState, GuiView, Save, WindowFocus},
};

/// main gui run function
pub(crate) fn run(rdr: &mut Records) -> anyhow::Result<Save> {
    rdr.0.sort_unstable();
    //rdr.reverse();

    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut state = GuiState {
        rdr,
        table_state: TableState::default().with_selected(Some(0)),
        view: GuiView::Normal,
        focus: WindowFocus::Table,
        changed_this_exection: HashSet::new(),
        search: None,
        add: None,
    };

    let save;
    loop {
        terminal.draw(|frame| {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    Constraint::Percentage(2),
                    Constraint::Percentage(91),
                    Constraint::Percentage(7),
                ])
                .split(frame.area());

            summarybar::draw(frame, layout[0], &state);
            table_window::draw(frame, layout[1], &mut state);
            searchbar::draw(frame, layout[2], &state);
            match &state.focus {
                WindowFocus::Table => {}
                WindowFocus::StageEdit(_, _) => status_edit_window::draw(frame, layout[1], &state),
                WindowFocus::Help => help_window::draw(frame, layout[1], &state),
                WindowFocus::Search => {}
                WindowFocus::Add => add_window::draw(frame, layout[1], &state),
                WindowFocus::Info => info_window::draw(frame, layout[1], &state),
            };
        })?;
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match state.focus {
                        WindowFocus::Table => {
                            if let ControlFlow::Break(s) =
                                table_window::handle_input(key, &mut state)
                            {
                                save = s;
                                break;
                            }
                        }
                        WindowFocus::StageEdit(_, _) => {
                            status_edit_window::handle_input(key, &mut state);
                        }
                        WindowFocus::Help => help_window::handle_input(key, &mut state),
                        WindowFocus::Search => searchbar::handle_input(key, &mut state),
                        WindowFocus::Add => add_window::handle_input(key, &mut state),
                        WindowFocus::Info => info_window::handle_input(key, &mut state),
                    };
                }
            }
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(save)
}

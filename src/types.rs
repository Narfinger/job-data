use std::{collections::HashSet, sync::LazyLock};

use anyhow::Context;
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    widgets::TableState,
};
use serde::{Deserialize, Serialize};
use time::{
    format_description::{self, BorrowedFormatItem},
    OffsetDateTime, UtcOffset,
};
use yansi::{Paint, Painted};

use crate::records::{Record, Records};

/// Time format
pub(crate) static FORMAT: LazyLock<Vec<BorrowedFormatItem<'_>>> =
    LazyLock::new(|| format_description::parse("[day]-[month]-[year]").expect("error"));

/// The local time
pub(crate) static NOW: LazyLock<OffsetDateTime> = LazyLock::new(|| {
    OffsetDateTime::now_local()
        .context("Cannot get now")
        .expect("Error")
});
/// The current offset
pub(crate) static CURRENT_OFFSET: LazyLock<UtcOffset> = LazyLock::new(|| {
    UtcOffset::current_local_offset()
        .context("Could not get offset")
        .expect("Error")
});
/// the date string of now
pub(crate) static DATE_STRING: LazyLock<String> =
    LazyLock::new(|| NOW.date().format(&FORMAT).expect("Error"));

/// Status of a job application
#[derive(Clone, Debug, Deserialize, Hash, Serialize, PartialEq, Eq)]
pub(crate) enum Status {
    /// we need to do something
    Todo,
    /// we are waiting for an update
    Pending,
    /// we got rejected
    Rejected,
    /// we declined the offer
    Declined,
}

/// simple display function for status on cmd
impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Todo => f.write_str("Todo"),
            Status::Pending => f.write_str("Pending"),
            Status::Rejected => f.write_str("Rejected"),
            Status::Declined => f.write_str("Declined"),
        }
    }
}

impl Status {
    /// display string for status
    pub(crate) fn print(&self) -> Painted<&str> {
        match self {
            Status::Todo => "TODO".red(),
            Status::Pending => "Pending".yellow(),
            Status::Declined => "Declined".green(),
            Status::Rejected => "Rejected".green(),
        }
    }

    /// function to toggle status in order, returning the new status
    #[must_use]
    pub(crate) fn next(&self) -> Status {
        match self {
            Status::Todo => Status::Pending,
            Status::Pending => Status::Rejected,
            Status::Rejected => Status::Todo,
            Status::Declined => Status::Todo,
        }
    }
}

/// Find the center rectangle
pub(crate) fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

/// Which part of jobs we show
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GuiView {
    /// Show only non-old Pending and Todo jobs
    Normal,
    /// Show only Pending jobs
    Old,
    /// Show all jobs
    All,
}

impl GuiView {
    /// toggle the guiview
    pub(crate) fn next(&mut self) -> GuiView {
        match self {
            GuiView::Normal => GuiView::Old,
            GuiView::Old => GuiView::All,
            GuiView::All => GuiView::Normal,
        }
    }
}

/// state of the tui
pub(crate) struct GuiState<'a> {
    /// The records
    pub(crate) rdr: &'a mut Records,
    /// The main table state
    pub(crate) table_state: TableState,
    /// Which parts of job we show
    pub(crate) view: GuiView,
    /// which window we have in focus
    pub(crate) focus: WindowFocus,
    /// record the index of all things we changed today so that we still show them
    pub(crate) changed_this_exection: HashSet<usize>,
    /// Are we searching something
    pub(crate) search: Option<String>,
    /// A job we want to add
    pub(crate) add: Option<AddStruct>,
}

impl<'a> GuiState<'a> {
    /// the filter function of which ones to show
    pub(crate) fn filter(&self, index: &usize, r: &Record) -> bool {
        let normal_filtering = self.search.as_ref().map(|s| s.is_empty()).unwrap_or(true);
        if normal_filtering {
            r.status == Status::Todo
                || self.changed_this_exection.contains(index)
                || match self.view {
                    GuiView::Normal => r.status == Status::Pending && !r.is_old(),
                    GuiView::Old => r.status == Status::Todo || r.status == Status::Pending,
                    GuiView::All => true,
                }
        } else {
            let search_string = &self.search.as_ref().unwrap();
            r.name.contains(*search_string)
        }
    }

    /// get the index in the record vector from the selection of the table
    pub(crate) fn get_real_index(&self) -> usize {
        let index = self.table_state.selected().unwrap();
        self.rdr
            .0
            .iter()
            .enumerate()
            .filter(|(index, r)| self.filter(index, r))
            .nth(index)
            .map(|(i, _)| i)
            .unwrap()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum AddFocusField {
    Company,
    JobName,
    Place,
}

impl AddFocusField {
    pub(crate) fn next(&self) -> AddFocusField {
        match self {
            AddFocusField::Company => AddFocusField::JobName,
            AddFocusField::JobName => AddFocusField::Place,
            AddFocusField::Place => AddFocusField::Company,
        }
    }
    pub(crate) fn prev(&self) -> AddFocusField {
        match self {
            AddFocusField::Company => AddFocusField::Place,
            AddFocusField::JobName => AddFocusField::Company,
            AddFocusField::Place => AddFocusField::JobName,
        }
    }
}

#[derive(Debug)]
pub(crate) struct AddStruct {
    pub(crate) company: String,
    pub(crate) jobname: String,
    pub(crate) place: String,
    pub(crate) focus: AddFocusField,
    pub(crate) modify: Option<usize>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum WindowFocus {
    /// The table
    Table,
    /// The edit stage popup
    StageEdit(String, usize),
    /// The help window
    Help,
    /// The search lower bar
    Search,
    /// The add popup
    Add,
    /// the info popup
    Info,
}

/// Should we save the records to disk or not
pub(crate) enum Save {
    /// Save the records to disk
    Save,
    /// Do not save the records to disk
    DoNotSave,
}

use std::{cmp::Ordering, collections::HashSet, sync::LazyLock};

use anyhow::Context;
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    widgets::TableState,
};
use serde::{Deserialize, Serialize};
use time::{
    format_description::{self, BorrowedFormatItem},
    Date, Duration, OffsetDateTime, UtcOffset,
};
use yansi::{Paint, Painted};

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

/// A record of a job application
#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct Record {
    /// the last time any action happened to this job
    pub(crate) last_action_date: String,
    /// the name of the company
    pub(crate) name: String,
    /// the job name
    pub(crate) subname: String,
    /// at what stage are we, i.e., first interview, second and so on
    pub(crate) stage: String,
    /// some additional information we want to store
    pub(crate) additional_info: String,
    /// the status of the job
    pub(crate) status: Status,
    /// where
    pub(crate) place: String,
}

impl PartialOrd for Record {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Record {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.status == Status::Todo && other.status != Status::Todo {
            Ordering::Less
        } else if self.status != Status::Todo && other.status == Status::Todo {
            Ordering::Greater
        } else {
            let my_date: Date = Date::parse(&self.last_action_date, &FORMAT).unwrap();
            let other_date: Date = Date::parse(&other.last_action_date, &FORMAT).unwrap();
            my_date.cmp(&other_date).reverse()
        }
    }
}

impl Record {
    /// cronstruct a new one
    pub(crate) fn new(company: String, jobname: String) -> Self {
        Record {
            name: company,
            subname: jobname,
            stage: String::new(),
            additional_info: String::new(),
            status: Status::Todo,
            last_action_date: DATE_STRING.clone(),
            place: String::new(),
        }
    }

    /// update the last action date
    fn update_date(&mut self) {
        self.last_action_date = DATE_STRING.clone();
    }

    /// toggle stage
    pub(crate) fn next_stage(&mut self) {
        self.status = self.status.next();
        self.update_date();
    }

    /// sets the status
    pub(crate) fn set_status(&mut self, status: Status) {
        self.status = status;
        self.update_date();
    }

    /// sets the stage of the job
    pub(crate) fn set_stage(&mut self, stage: String) {
        self.stage = stage;
        self.update_date();
    }

    /// test if the job is old, i.e., 2 weeks after last action date
    pub(crate) fn is_old(&self) -> bool {
        let d_primitive_date = Date::parse(&self.last_action_date, &FORMAT)
            .context("Cannot parse primitive date")
            .expect("Error");
        let d_primitive = d_primitive_date
            .with_hms(0, 0, 0)
            .context("Could not add time")
            .expect("Error");
        let d = d_primitive.assume_offset(*CURRENT_OFFSET);
        self.status != Status::Todo && *NOW - d >= Duration::weeks(2)
    }

    /// print one entry
    pub(crate) fn print(&self, index: usize, truncate: bool) -> anyhow::Result<()> {
        if truncate && self.is_old() {
            println!(
                "{:2} | {:-^10} | {:-^20} | {:-^20} | {:^37} | {:^30} | {}",
                index.dim(),
                self.status.print().dim(),
                self.last_action_date.dim(),
                self.name.bold().dim(),
                self.subname.bold().dim(),
                self.stage.dim(),
                self.place,
            );
        } else if truncate {
            println!(
                "{:2} | {:-^10} | {:-^20} | {:-^20} | {:^37} | {:^30} | {}",
                index,
                self.status.print(),
                self.last_action_date,
                self.name.bold(),
                self.subname.bold(),
                self.stage,
                self.place,
            );
        } else {
            println!(
                "{:2} | {:-^10} | {:-^20} | {:-^20} | {:^37} | {:^30} | {} | {}",
                index,
                self.status.print(),
                self.last_action_date,
                self.name.bold(),
                self.subname.bold(),
                self.stage,
                self.additional_info,
                self.place,
            );
        }
        Ok(())
    }
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
    pub(crate) rdr: &'a mut Vec<Record>,
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
}

#[derive(Debug)]
pub(crate) struct AddStruct {
    pub(crate) company: String,
    pub(crate) jobname: String,
    pub(crate) focus: AddFocusField,
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
}

/// Should we save the records to disk or not
pub(crate) enum Save {
    /// Save the records to disk
    Save,
    /// Do not save the records to disk
    DoNotSave,
}

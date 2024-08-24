use std::sync::LazyLock;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use time::{
    format_description::{self, BorrowedFormatItem},
    Date, Duration, OffsetDateTime, UtcOffset,
};
use yansi::{Paint, Painted};

pub(crate) static FORMAT: LazyLock<Vec<BorrowedFormatItem<'_>>> =
    LazyLock::new(|| format_description::parse("[day]-[month]-[year]").expect("error"));
pub(crate) static NOW: LazyLock<OffsetDateTime> = LazyLock::new(|| {
    OffsetDateTime::now_local()
        .context("Cannot get now")
        .expect("Error")
});
pub(crate) static CURRENT_OFFSET: LazyLock<UtcOffset> = LazyLock::new(|| {
    UtcOffset::current_local_offset()
        .context("Could not get offset")
        .expect("Error")
});
pub(crate) static DATE_STRING: LazyLock<String> =
    LazyLock::new(|| NOW.date().format(&FORMAT).expect("Error"));

#[derive(Debug)]
pub(crate) enum Save {
    Save,
    DoNotSave,
}

#[derive(Clone, Debug, Deserialize, Hash, Serialize, PartialEq, Eq)]
pub(crate) enum Status {
    Todo,
    Pending,
    Rejected,
    Declined,
}

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
    pub(crate) fn print(&self) -> Painted<&str> {
        match self {
            Status::Todo => "TODO".red(),
            Status::Pending => "Pending".yellow(),
            Status::Declined => "Declined".green(),
            Status::Rejected => "Rejected".green(),
        }
    }

    pub(crate) fn next(&self) {
        match self {
            Status::Todo => Status::Pending,
            Status::Pending => Status::Rejected,
            Status::Rejected => Status::Todo,
            Status::Declined => Status::Todo,
        };
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct Record {
    pub(crate) last_action_date: String,
    pub(crate) name: String,
    pub(crate) subname: String,
    pub(crate) stage: String,
    pub(crate) additional_info: String,
    pub(crate) status: Status,
}

impl Record {
    fn update_date(&mut self) {
        self.last_action_date = DATE_STRING.clone();
    }

    /// toggle stage
    pub(crate) fn next_stage(&mut self) {
        self.status.next();
        self.update_date();
    }

    /// sets the status
    pub(crate) fn set_status(&mut self, status: Status) {
        self.status = status;
        self.update_date();
    }

    pub(crate) fn set_stage(&mut self, stage: String) {
        self.stage = stage;
        self.update_date();
    }

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
        let mut r = self.additional_info.clone();
        if truncate {
            r.truncate(30);
        }
        if self.is_old() {
            println!(
                "{:2} | {:-^10} | {:-^20} | {:-^20} | {:^37} | {:^30} | {}",
                index.dim(),
                self.status.print().dim(),
                self.last_action_date.dim(),
                self.name.bold().dim(),
                self.subname.bold().dim(),
                self.stage.dim(),
                r.dim(),
            );
        } else {
            println!(
                "{:2} | {:-^10} | {:-^20} | {:-^20} | {:^37} | {:^30} | {}",
                index,
                self.status.print(),
                self.last_action_date,
                self.name.bold(),
                self.subname.bold(),
                self.stage,
                r,
            );
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GuiView {
    Normal,
    Old,
    All,
}

impl GuiView {
    pub(crate) fn next(&mut self) -> GuiView {
        match self {
            GuiView::Normal => GuiView::Old,
            GuiView::Old => GuiView::All,
            GuiView::All => GuiView::Normal,
        }
    }
}

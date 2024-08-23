use anyhow::Context;
use serde::{Deserialize, Serialize};
use time::{format_description, Date, Duration, OffsetDateTime, UtcOffset};
use yansi::{Paint, Painted};

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
        let format =
            format_description::parse("[day]-[month]-[year]").expect("Error in format description");
        let date = OffsetDateTime::now_utc()
            .date()
            .format(&format)
            .expect("error in time");
        self.last_action_date = date;
    }

    /// toggle stage
    pub(crate) fn next_stage(&mut self) {
        self.status = match self.status {
            Status::Todo => Status::Pending,
            Status::Pending => Status::Rejected,
            Status::Rejected => Status::Todo,
            Status::Declined => Status::Todo,
        };
        self.update_date();
    }

    /// sets the status
    pub(crate) fn set_status(&mut self, status: Status) {
        self.status = status;
        self.update_date();
    }

    /// print one entry
    pub(crate) fn print(
        &self,
        index: usize,
        truncate: bool,
        current_offset: UtcOffset,
        now: OffsetDateTime,
    ) -> anyhow::Result<()> {
        let format = format_description::parse("[day]-[month]-[year]")
            .context("time format description error")?;
        let mut r = self.additional_info.clone();
        if truncate {
            r.truncate(30);
        }
        let d_primitive_date =
            Date::parse(&self.last_action_date, &format).context("Cannot parse primitive date")?;
        let d_primitive = d_primitive_date
            .with_hms(0, 0, 0)
            .context("Could not add time")?;
        let d = d_primitive.assume_offset(current_offset);
        if self.status != Status::Todo && now - d >= Duration::weeks(2) {
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

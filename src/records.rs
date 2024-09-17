use std::{
    cmp::Ordering,
    fs::File,
    io::{BufReader, BufWriter},
};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use time::{Date, Duration};
use yansi::Paint;

use crate::{
    types::{Status, CURRENT_OFFSET, DATE_STRING, FORMAT, NOW},
    PATH,
};

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
    pub(crate) fn new(company: String, jobname: String, place: String) -> Self {
        Record {
            name: company,
            subname: jobname,
            stage: String::new(),
            additional_info: String::new(),
            status: Status::Todo,
            last_action_date: DATE_STRING.clone(),
            place,
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

pub(crate) struct Records(pub(crate) Vec<Record>);

impl Records {
    /// load records
    pub(crate) fn load() -> anyhow::Result<Self> {
        let f = File::open(PATH.clone())?;
        let br = BufReader::new(f);
        let rdr = csv::Reader::from_reader(br)
            .deserialize()
            .map(|r| r.unwrap())
            .collect::<Vec<Record>>();
        let rej = rdr
            .iter()
            .filter(|a| a.status == Status::Declined || a.status == Status::Rejected);
        let pen = rdr.iter().filter(|a| a.status == Status::Pending);
        let todo = rdr.iter().filter(|a| a.status == Status::Todo);
        Ok(Records(
            rej.chain(pen).chain(todo).cloned().collect::<Vec<Record>>(),
        ))
    }

    /// write records to file
    pub(crate) fn write(&self) -> anyhow::Result<()> {
        let f = File::create(PATH.clone())?;
        let br = BufWriter::new(f);
        let mut wtr = csv::Writer::from_writer(br);
        for i in &self.0 {
            wtr.serialize(i)?;
        }
        wtr.flush()?;
        Ok(())
    }

    pub(crate) fn get<'a>(&'a self, index: usize) -> Option<&'a Record> {
        self.0.get(index)
    }

    pub(crate) fn get_mut<'a>(&'a mut self, index: usize) -> Option<&'a mut Record> {
        self.0.get_mut(index)
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn iter<'a>(&'a self) -> std::slice::Iter<'_, Record> {
        self.0.iter()
    }
}

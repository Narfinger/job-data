use std::{
    cmp::Ordering, fs::File, io::{BufReader, BufWriter}
};
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, FromInto};
use time::{Date, Duration, OffsetDateTime};
use yansi::Paint;

use crate::{
    types::{Status, FORMAT},
    PATH,
};

time::serde::format_description!(my_format, Date,"[day]-[month]-[year]");
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
struct MyDate(#[serde(with = "my_format")] Date);

impl From<Date> for MyDate {
    fn from(value: Date) -> Self {
        MyDate(value)
    }
}

impl From<MyDate> for Date {
    fn from(value: MyDate) -> Self {
        value.0
    }
}



/// A record of a job application
#[serde_as]
#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct Record {
    /// the last time any action happened to this job we push new strings to the back!
    #[serde_as(as = "Vec<FromInto<MyDate>>")]
    last_action_date: Vec<Date>,
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
            self.last_action_date.first().unwrap().cmp(other.last_action_date.first().unwrap()).reverse()
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
            last_action_date: vec![OffsetDateTime::now_local().expect("Error in getting time").date()],
            place,
        }
    }

    /// update the last action date
    pub(crate) fn update_date(&mut self) {
        let now = OffsetDateTime::now_local().expect("Error in getting time").date();
        self.last_action_date.push(now);
    }

    /// returns the date
    pub(crate) fn get_date(&self) -> &Date {
        self.last_action_date.last().unwrap()
    }

    /// returns the last date we had an action formatted
    pub(crate) fn date_string(&self) -> String {
        let d = self.last_action_date.last().unwrap();
        d.format(&FORMAT).unwrap()
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
        let last_time = self.last_action_date.first();
        let today = OffsetDateTime::now_local().expect("Error in getting time").date();
        if let Some(l) = last_time {
            self.status != Status::Todo && today - *l >= Duration::weeks(2)
        } else {
            false
        }

    }

    /// print one entry
    pub(crate) fn print(&self, index: usize, truncate: bool) -> anyhow::Result<()> {
        let date = self.last_action_date.first().unwrap().format(&FORMAT)?;
        if truncate && self.is_old() {
            println!(
                "{:2} | {:-^10} | {:-^20} | {:-^20} | {:^37} | {:^30} | {}",
                index.dim(),
                self.status.print().dim(),
                date.dim(),
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
                date,
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
                date,
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
        let d = &mut serde_json::Deserializer::from_reader(br);

        let result: Result<Vec<Record>, _> = serde_path_to_error::deserialize(d);
        if let Err(e) = result {
            Err(anyhow!("Error in parsing {}", e))
        } else {

            let rdr = result.unwrap();
            let rej = rdr
            .iter()
            .filter(|a| a.status == Status::Declined || a.status == Status::Rejected);
        let pen = rdr.iter().filter(|a| a.status == Status::Pending);
        let todo = rdr.iter().filter(|a| a.status == Status::Todo);
        Ok(Records(
            rej.chain(pen).chain(todo).cloned().collect::<Vec<Record>>(),
        ))
    }
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

    pub(crate) fn get(&self, index: usize) -> Option<&Record> {
        self.0.get(index)
    }

    pub(crate) fn get_mut(&mut self, index: usize) -> Option<&mut Record> {
        self.0.get_mut(index)
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn iter(&self) -> std::slice::Iter<'_, Record> {
        self.0.iter()
    }
}

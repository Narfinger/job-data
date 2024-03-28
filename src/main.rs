use ansi_term::{
    ANSIGenericString,
    Colour::{Green, Red, Yellow},
};
use clap::{arg, command, Parser, Subcommand};
use csv::DeserializeRecordsIter;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, BufWriter},
};

const PATH: &str = "/home/engelzz/Documents/job-applications.csv";

#[derive(Debug, Deserialize, Serialize)]
enum Status {
    Todo,
    Pending,
    Rejected,
    Declined,
}

impl Status {
    fn print(&self) -> ANSIGenericString<str> {
        match self {
            Status::Todo => Red.paint("TODO    "),
            Status::Pending => Yellow.paint("Pending "),
            Status::Declined => Green.paint("Declined"),
            Status::Rejected => Green.paint("Rejected"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
struct Record {
    last_action_date: String,
    name: String,
    stage: String,
    additional_info: String,
    status: Status,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// change the status to pending of input
    #[arg(short, long)]
    pending: Option<usize>,

    /// change the status to rejected of input
    #[arg(short, long)]
    rejected: Option<usize>,
}

fn print(rdr: &[Record]) -> anyhow::Result<()> {
    for (i, result) in rdr.iter().enumerate() {
        let record = result;
        println!(
            "{} | {:-^20} | {:-^20} | {:-^20} | {} | {}",
            i,
            record.status.print(),
            record.last_action_date,
            record.name,
            record.stage,
            record.additional_info,
        );
    }
    Ok(())
}

fn write(rdr: &[Record]) -> anyhow::Result<()> {
    let f = File::create(PATH)?;
    let br = BufWriter::new(f);
    let mut wtr = csv::Writer::from_writer(br);
    for i in rdr {
        wtr.serialize(i)?;
    }
    wtr.flush()?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let f = File::open(PATH)?;
    let br = BufReader::new(f);
    let mut rdr = csv::Reader::from_reader(br)
        .deserialize()
        .map(|r| r.unwrap())
        .collect::<Vec<Record>>();

    if let Some(i) = cli.pending {
        rdr.get_mut(i).unwrap().status = Status::Pending;
        write(&rdr)?;
    }
    if let Some(i) = cli.rejected {
        rdr.get_mut(i).unwrap().status = Status::Rejected;
        write(&rdr)?;
    }

    print(&rdr)?;

    Ok(())
}

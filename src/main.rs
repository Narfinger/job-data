use anyhow::Context;
use clap::builder::TypedValueParser as _;
use clap::{arg, command, Parser};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    io::{BufReader, BufWriter},
};
use time::{format_description, Date, OffsetDateTime};
use yansi::{Paint, Painted};

const PATH: &str = "/home/engelzz/Documents/job-applications.csv";

#[derive(Clone, Debug, Deserialize, Hash, Serialize, PartialEq, Eq)]
enum Status {
    Todo,
    Pending,
    Rejected,
    Declined,
}

impl Status {
    fn print(&self) -> Painted<&str> {
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
struct Record {
    last_action_date: String,
    name: String,
    stage: String,
    additional_info: String,
    status: Status,
}

fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    let pos = s
        .find(' ')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

#[derive(Parser, Debug)]
#[command(version, about, long_about)]
struct Cli {
    /// change the status to pending of input
    #[arg(short, long, value_name = "index")]
    pending: Option<usize>,

    /// change the status to rejected of input
    #[arg(short, long, value_name = "index")]
    rejected: Option<usize>,

    /// open the file in editor
    #[arg(short, long)]
    open: bool,

    /// Status to change
    #[arg(long, num_args=2, value_names = ["index", "Status"])]
    status_change: Option<Vec<String>>,

    // add new job status
    #[arg(short, long, num_args = 2, value_names = ["Company Name", "Stage"])]
    add: Option<Vec<String>>,

    // search for a company
    #[arg(short, long)]
    search: Option<String>,
}

fn print(rdr: &[Record]) -> anyhow::Result<()> {
    print_stats(rdr)?;
    for (i, result) in rdr.iter().enumerate() {
        let record = result;
        println!(
            "{:2} | {:-^20} | {:-^20} | {:^20} | {:^30} | {}",
            i,
            record.status.print(),
            record.last_action_date,
            record.name.bold(),
            record.stage,
            record.additional_info,
        );
    }
    Ok(())
}

fn print_stats(rdr: &[Record]) -> anyhow::Result<()> {
    let vals = rdr.iter().fold(HashMap::new(), |mut red, elem| {
        let val = red.get(&elem.status).unwrap_or(&0);
        red.insert(&elem.status, val + 1);
        red
    });
    println!("-------------------STATS-------------------------------------");
    for (key, val) in vals.iter() {
        let key_print = key.print();
        print!("{}: {}/{} |", key_print, val, rdr.len());
    }
    println!("\n-------------------------------------------------------------");
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

    let mut rdr = {
        let f = File::open(PATH)?;
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
        rej.chain(pen).chain(todo).cloned().collect::<Vec<Record>>()
    };
    let format = format_description::parse("[day]-[month]-[year]")?;
    let date = OffsetDateTime::now_utc().date().format(&format)?;

    if let Some(i) = cli.pending {
        let index = i;
        rdr.get_mut(index).unwrap().status = Status::Pending;
        rdr.get_mut(index).unwrap().last_action_date = date;
        write(&rdr)?;
    } else if let Some(i) = cli.rejected {
        rdr.get_mut(i).unwrap().status = Status::Rejected;
        rdr.get_mut(i).unwrap().last_action_date = date;
        write(&rdr)?;
    } else if let Some(v) = cli.status_change {
        if let Ok(index) = v.first().unwrap().parse::<usize>() {
            rdr.get_mut(index).unwrap().stage = v.get(1).unwrap().to_string();
            write(&rdr)?;
        } else {
            println!("Not a valid integer");
        }
    } else if cli.open {
        open::that(PATH).context("Could not open file")?;
    } else if let Some(v) = cli.add {
        println!("Would add {:?}", v);
        let r = Record {
            last_action_date: date,
            name: v.first().unwrap().clone(),
            stage: v.get(1).unwrap().clone(),
            additional_info: "".to_string(),
            status: Status::Todo,
        };
        rdr.push(r);
        write(&rdr)?;
        print(&rdr)?;
        return Ok(());
    } else if let Some(c) = cli.search {
        let res = rdr
            .into_iter()
            .filter(|r| r.name.contains(&c))
            .collect::<Vec<Record>>();
        print(&res)?;
        return Ok(());
    }

    print(&rdr)?;

    Ok(())
}

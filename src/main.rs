use anyhow::Context;
use clap::{arg, command, Parser};
use inquire::Confirm;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, BufWriter},
};
use time::{format_description, Date, Duration, OffsetDateTime, UtcOffset};
use types::{Record, Status};
use yansi::{Paint, Painted};

mod gui;
mod types;

const PATH: &str = "/home/engelzz/Documents/job-applications.csv";

#[derive(Parser, Debug)]
#[command(version, about, long_about)]
struct Cli {
    /// show all values
    #[arg(long)]
    all: bool,

    /// change the status to pending of input
    #[arg(short, long, value_name = "index")]
    pending: Option<usize>,

    /// change the status to rejected of input
    #[arg(short, long, value_name = "index")]
    rejected: Option<usize>,

    /// set to todo
    #[arg(short, long, value_name = "index")]
    todo: Option<usize>,

    /// show full entry for one
    #[arg(short, long, value_name = "index")]
    info: Option<usize>,

    /// open the file in editor
    #[arg(short, long)]
    open: bool,

    /// Info to change
    #[arg(long, num_args=2, value_names = ["index", "Info"])]
    info_change: Option<Vec<String>>,

    /// stage  to change
    #[arg(long, num_args=2, value_names = ["index", "Stage"])]
    stage_change: Option<Vec<String>>,

    /// add new job status
    #[arg(short, long, num_args = 2..=3, value_names = ["Company Name", "Sub Name", "Additional Info"])]
    add: Option<Vec<String>>,

    /// search for a company
    #[arg(short, long)]
    search: Option<String>,

    /// open the tui
    #[arg(long)]
    tui: bool,
}

/// print one entry
fn print_record(
    i: usize,
    record: &Record,
    truncate: bool,
    current_offset: UtcOffset,
    now: OffsetDateTime,
) -> anyhow::Result<()> {
    let format = format_description::parse("[day]-[month]-[year]")
        .context("time format description error")?;
    let mut r = record.additional_info.clone();
    if truncate {
        r.truncate(30);
    }
    let d_primitive_date =
        Date::parse(&record.last_action_date, &format).context("Cannot parse primitive date")?;
    let d_primitive = d_primitive_date
        .with_hms(0, 0, 0)
        .context("Could not add time")?;
    let d = d_primitive.assume_offset(current_offset);
    if record.status != Status::Todo && now - d >= Duration::weeks(2) {
        println!(
            "{:2} | {:-^10} | {:-^20} | {:-^20} | {:^37} | {:^30} | {}",
            i.dim(),
            record.status.print().dim(),
            record.last_action_date.dim(),
            record.name.bold().dim(),
            record.subname.bold().dim(),
            record.stage.dim(),
            r.dim(),
        );
    } else {
        println!(
            "{:2} | {:-^10} | {:-^20} | {:-^20} | {:^37} | {:^30} | {}",
            i,
            record.status.print(),
            record.last_action_date,
            record.name.bold(),
            record.subname.bold(),
            record.stage,
            r,
        );
    }
    Ok(())
}

/// print all entries
fn print(rdr: &[Record], truncate: bool, show_all: bool) -> anyhow::Result<()> {
    print_stats(rdr)?;
    println!(
        "{:2} | {:^10} | {:^20} | {:^20} | {:^37} | {:^30} | {}",
        "",
        "Status".underline(),
        "Last Date".underline(),
        "Name".underline(),
        "Subname".underline(),
        "Stage".underline(),
        "Info".underline()
    );

    let now = OffsetDateTime::now_local().context("Cannot get now")?;
    let current_offset = UtcOffset::current_local_offset().context("Could not get offset")?;
    for (i, record) in rdr.iter().enumerate() {
        // we want to keep the record numbers the same
        if show_all || record.status == Status::Pending || record.status == Status::Todo {
            print_record(i, record, truncate, current_offset, now)?;
        }
    }
    Ok(())
}

/// print the stats
fn print_stats(rdr: &[Record]) -> anyhow::Result<()> {
    let vals = rdr.iter().fold(HashMap::new(), |mut red, elem| {
        let val = red.get(&elem.status).unwrap_or(&0);
        red.insert(&elem.status, val + 1);
        red
    });
    println!("-------------------STATS----------------------------------------");
    for (key, val) in vals.iter() {
        let key_print = key.print();
        let percentage: f64 = (*val as f64) / (rdr.len() as f64);
        print!(
            "{}: {}/{} ({:.2}%)| ",
            key_print,
            val,
            rdr.len(),
            percentage
        );
    }
    println!("\n----------------------------------------------------------------");
    Ok(())
}

/// write records to file
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

/// ask if we should change the status
fn ask_if_change_status(rdr: &[Record], index: usize, new_stage: &Status) -> bool {
    let rec = rdr.get(index).unwrap();
    let ans = Confirm::new(&format!(
        "Do you want to change {} | {} from {} to {}",
        rec.name, rec.subname, rec.status, new_stage
    ))
    .with_default(false)
    .prompt();

    match ans {
        Ok(true) => true,
        Ok(false) => false,
        Err(_) => false,
    }
}

/// ask if we should change
fn ask_if_change(rdr: &[Record], index: usize) -> bool {
    let rec = rdr.get(index).unwrap();
    let ans = Confirm::new(&format!(
        "Do you want to change {} | {}",
        rec.name, rec.subname,
    ))
    .with_default(false)
    .prompt();

    match ans {
        Ok(true) => true,
        Ok(false) => false,
        Err(_) => false,
    }
}

fn change_status(rdr: &mut [Record], index: usize, status: Status) -> anyhow::Result<()> {
    if ask_if_change_status(rdr, index, &status) {
        let format = format_description::parse("[day]-[month]-[year]")?;
        let date = OffsetDateTime::now_utc().date().format(&format)?;
        rdr.get_mut(index).unwrap().status = status;
        rdr.get_mut(index).unwrap().last_action_date = date;
        write(rdr)?;
    }
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

    if let Some(i) = cli.pending {
        change_status(&mut rdr, i, Status::Pending)?;
    } else if let Some(i) = cli.rejected {
        change_status(&mut rdr, i, Status::Rejected)?;
    } else if let Some(i) = cli.todo {
        change_status(&mut rdr, i, Status::Todo)?;
    } else if let Some(v) = cli.info_change {
        let format = format_description::parse("[day]-[month]-[year]")?;
        let date = OffsetDateTime::now_utc().date().format(&format)?;
        if let Ok(i) = v.first().unwrap().parse::<usize>() {
            println!(
                "Chainging from {} to {}",
                rdr.get(i).unwrap().additional_info,
                &v.get(1).unwrap().to_string(),
            );
            if ask_if_change(&rdr, i) {
                rdr.get_mut(i).unwrap().additional_info = v.get(1).unwrap().to_string();
                rdr.get_mut(i).unwrap().last_action_date = date;
                write(&rdr)?;
            }
        } else {
            println!("Not a valid integer");
        }
    } else if let Some(v) = cli.stage_change {
        let format = format_description::parse("[day]-[month]-[year]")?;
        let date = OffsetDateTime::now_utc().date().format(&format)?;
        if let Ok(i) = v.first().unwrap().parse::<usize>() {
            println!(
                "Chainging from {} to {}",
                rdr.get(i).unwrap().stage,
                &v.get(1).unwrap(),
            );
            if ask_if_change(&rdr, i) {
                rdr.get_mut(i).unwrap().stage = v.get(1).unwrap().to_string();
                rdr.get_mut(i).unwrap().last_action_date = date;
                write(&rdr)?;
            }
        } else {
            println!("Not a valid integer");
        }
    } else if cli.open {
        open::that(PATH).context("Could not open file")?;
    } else if let Some(v) = cli.add {
        let format = format_description::parse("[day]-[month]-[year]")?;
        let date = OffsetDateTime::now_utc().date().format(&format)?;
        let r = Record {
            last_action_date: date,
            name: v.first().unwrap().clone(),
            subname: v.get(1).unwrap().clone(),
            stage: "Pending".to_string(),
            additional_info: v.get(2).unwrap_or(&String::new()).clone(),
            status: Status::Todo,
        };
        rdr.push(r);
        write(&rdr)?;
        print(&rdr, true, true)?;
        return Ok(());
    } else if let Some(c) = cli.search {
        let res = rdr
            .into_iter()
            .filter(|r| r.name.contains(&c))
            .collect::<Vec<Record>>();
        print(&res, false, true)?;
        return Ok(());
    } else if let Some(c) = cli.info {
        if let Some(res) = rdr.get(c) {
            let r = vec![res.clone()];
            print(&r, false, true)?;
        } else {
            println!("Could not find record");
        }
        return Ok(());
    } else if cli.tui {
        gui::run(&mut rdr);
        return Ok(());
    }

    print(&rdr, true, cli.all)?;

    Ok(())
}

use anyhow::Context;
use clap::{arg, command, Parser};
use inquire::Confirm;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
    sync::LazyLock,
};
use types::{Record, Save, Status, DATE_STRING};
use yansi::Paint;

mod add_window;
mod gui;
mod help_window;
mod searchbar;
mod status_edit_window;
mod summarybar;
mod table_window;
mod types;

static PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let u = directories::UserDirs::new().expect("Cannot find userdirs");
    u.document_dir().unwrap().join("job-applications.csv")
});

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

/// print all entries
fn print(rdr: &[Record], truncate: bool, show_all: bool) -> anyhow::Result<()> {
    print_stats(rdr)?;
    if truncate {
        println!(
            "{:2} | {:^10} | {:^20} | {:^20} | {:^37} | {:^30} | {}",
            "",
            "Status".underline(),
            "Last Date".underline(),
            "Name".underline(),
            "Subname".underline(),
            "Stage".underline(),
            "Place".underline(),
        );
    } else {
        println!(
            "{:2} | {:^10} | {:^20} | {:^20} | {:^37} | {:^30} | {:^20} | {}",
            "",
            "Status".underline(),
            "Last Date".underline(),
            "Name".underline(),
            "Subname".underline(),
            "Stage".underline(),
            "Info".underline(),
            "Place".underline(),
        );
    }

    for (i, record) in rdr.iter().enumerate() {
        // we want to keep the record numbers the same
        if show_all || record.status == Status::Pending || record.status == Status::Todo {
            record.print(i, truncate)?;
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
            "{}: {}/{} ({:.1}%)| ",
            key_print,
            val,
            rdr.len(),
            percentage * 100_f64
        );
    }
    println!("\n----------------------------------------------------------------");
    Ok(())
}

/// write records to file
fn write(rdr: &[Record]) -> anyhow::Result<()> {
    let f = File::create(PATH.clone())?;
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
        rdr.get_mut(index).unwrap().set_status(status);
        write(rdr)?;
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let mut rdr = {
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
        rej.chain(pen).chain(todo).cloned().collect::<Vec<Record>>()
    };

    if let Some(i) = cli.pending {
        change_status(&mut rdr, i, Status::Pending)?;
    } else if let Some(i) = cli.rejected {
        change_status(&mut rdr, i, Status::Rejected)?;
    } else if let Some(i) = cli.todo {
        change_status(&mut rdr, i, Status::Todo)?;
    } else if let Some(v) = cli.info_change {
        if let Ok(i) = v.first().unwrap().parse::<usize>() {
            println!(
                "Chainging from {} to {}",
                rdr.get(i).unwrap().additional_info,
                &v.get(1).unwrap().to_string(),
            );
            if ask_if_change(&rdr, i) {
                rdr.get_mut(i).unwrap().additional_info = v.get(1).unwrap().to_string();
                rdr.get_mut(i).unwrap().last_action_date = DATE_STRING.clone();
                write(&rdr)?;
            }
        } else {
            println!("Not a valid integer");
        }
    } else if let Some(v) = cli.stage_change {
        if let Ok(i) = v.first().unwrap().parse::<usize>() {
            println!(
                "Chainging from {} to {}",
                rdr.get(i).unwrap().stage,
                &v.get(1).unwrap(),
            );
            if ask_if_change(&rdr, i) {
                rdr.get_mut(i)
                    .unwrap()
                    .set_stage(v.get(1).unwrap().to_string());
                write(&rdr)?;
            }
        } else {
            println!("Not a valid integer");
        }
    } else if cli.open {
        open::that(PATH.clone()).context("Could not open file")?;
    } else if let Some(v) = cli.add {
        let r = Record {
            last_action_date: DATE_STRING.clone(),
            name: v.first().unwrap().clone(),
            subname: v.get(1).unwrap().clone(),
            stage: "Pending".to_string(),
            additional_info: v.get(2).unwrap_or(&String::new()).clone(),
            status: Status::Todo,
            place: String::new(),
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
        match gui::run(&mut rdr)? {
            Save::Save => {
                println!("Writing");
                write(&rdr)?;
            }
            Save::DoNotSave => {
                println!("We did not save");
            }
        }
        return Ok(());
    }

    print(&rdr, true, cli.all)?;

    Ok(())
}

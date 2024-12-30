use clap::Parser;
use chrono::{Local, NaiveDate, NaiveDateTime};
use std::str::FromStr;
use std::thread;
use std::time::Duration;
use std::io::Write; // Required for stdout.flush()

/// Simple CLI program to process date and time
#[derive(Parser)]
struct Cli {
    /// Date in the format YYYY-MM-DD (optional, defaults to today)
    #[clap(short, long)]
    date: Option<String>,

    /// Time in the format HH:MM:SS (required)
    #[clap(short, long)]
    time: String,
}

fn parse_time(time: &str) -> Result<(u32, u32, u32), String> {
    let parts: Vec<&str> = time.split(':').collect();
    match parts.len() {
        3 => {
            let hours = u32::from_str(parts[0]).map_err(|_| "Invalid hour")?;
            let minutes = u32::from_str(parts[1]).map_err(|_| "Invalid minute")?;
            let seconds = u32::from_str(parts[2]).map_err(|_| "Invalid second")?;
            Ok((hours, minutes, seconds))
        }
        2 => {
            let hours = u32::from_str(parts[0]).map_err(|_| "Invalid hour")?;
            let minutes = u32::from_str(parts[1]).map_err(|_| "Invalid minute")?;
            Ok((hours, minutes, 0))
        }
        1 => {
            let hours = u32::from_str(parts[0]).map_err(|_| "Invalid hour")?;
            Ok((hours, 0, 0))
        }
        _ => Err("Invalid time format, use HH[:MM[:SS]]".to_string()),
    }
}

fn validate_datetime(date: NaiveDate, time: &str) -> NaiveDateTime {
    match parse_time(time) {
        Ok((hours, minutes, seconds)) => date.and_hms_opt(hours, minutes, seconds).expect("Invalid time"),
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    }
}

fn main() {
    let args = Cli::parse();

    // Get today's date if no date is provided
    let date = match args.date {
        Some(ref d) => NaiveDate::parse_from_str(d, "%Y-%m-%d").expect("Invalid date format, use YYYY-MM-DD"),
        None => Local::now().naive_local().into(),
    };

    // Validate and combine date and time
    let target_datetime = validate_datetime(date, &args.time);

    let now = Local::now().naive_local();
    if target_datetime <= now {
        eprintln!("Target date/time must be in the future");
        std::process::exit(1);
    }

    let total_duration = target_datetime - now;
    let total_seconds = total_duration.num_seconds();

    for elapsed in 0..=total_seconds {
        let percentage = (elapsed as f64 / total_seconds as f64) * 100.0;
        print!("\rEndzeit: {} [{:<50}] {:.2}%", target_datetime, "=".repeat((percentage / 2.0) as usize), percentage);
        std::io::stdout().flush().unwrap();
        thread::sleep(Duration::from_secs(1));
    }

    println!("\nEndzeit reached!");
}

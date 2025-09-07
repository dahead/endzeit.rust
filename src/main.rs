use clap::Parser;
use chrono::{Local, NaiveDate, NaiveDateTime};
use std::str::FromStr;
use std::thread;
use std::time::Duration;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Gauge};
use ratatui::backend::CrosstermBackend;
use crossterm::{ event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}, };
use std::process::Command;
use std::io;

#[derive(Parser)]
struct Cli {
    /// Date in the format YYYY-MM-DD (optional, defaults to today)
    #[clap(short, long)]
    date: Option<String>,

    /// Time in the format HH:MM:SS (optional, defaults to current time if not provided)
    #[clap(short, long)]
    time: Option<String>,

    /// Command to execute when endzeit finishes
    #[clap(long)]
    execute: Option<String>,
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

fn validate_datetime(date: NaiveDate, time: Option<&str>) -> NaiveDateTime {
    let now = Local::now().naive_local();
    match time {
        Some(t) => {
            match parse_time(t) {
                Ok((hours, minutes, seconds)) => date.and_hms_opt(hours, minutes, seconds).expect("Invalid time"),
                Err(err) => {
                    eprintln!("{}", err);
                    std::process::exit(1);
                }
            }
        },
        None => now,
    }
}

fn execute_file(command_with_args: &str) -> io::Result<()> {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", command_with_args])
            .output()?
    } else {
        Command::new("sh")
            .args(["-c", command_with_args])
            .output()?
    };

    if output.status.success() {
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    } else {
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}
fn main() {
    let args = Cli::parse();

    // Get today's date if no date is provided
    let date = match args.date {
        Some(ref d) => NaiveDate::parse_from_str(d, "%Y-%m-%d").expect("Invalid date format, use YYYY-MM-DD"),
        None => Local::now().naive_local().into(),
    };

    // Validate and combine date and time
    let target_datetime = validate_datetime(date, args.time.as_deref());

    let now = Local::now().naive_local();
    if target_datetime <= now {
        eprintln!("Target date/time must be in the future");
        std::process::exit(1);
    }

    // --- Set up ratatui terminal -----------------------------------------
    enable_raw_mode().expect("Failed to enable raw mode");
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).expect("Failed to enter alternate screen");
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).expect("Failed to create terminal");
    // -----------------------------------------------------------------------

    let total_duration = target_datetime - now;
    let total_seconds = total_duration.num_seconds() as f64;
    let start_instant = std::time::Instant::now();

    // Main UI loop
    loop {
        // Calculate elapsed time
        let elapsed = start_instant.elapsed().as_secs_f64();
        let remaining = if total_seconds - elapsed > 0.0 {
            total_seconds - elapsed
        } else {
            0.0
        };

        // Calculate progress percentage
        let percentage = if total_seconds > 0.0 {
            (elapsed / total_seconds).min(1.0) * 100.0
        } else {
            100.0
        };

        // --- Render UI ----------------------------------------------------
        terminal.draw(|f| {
            let size = f.area();

            // Create a block for the overall layout
            let block = Block::default()
                .title(format!("Endzeit: {}", target_datetime))
                .borders(Borders::ALL);
            f.render_widget(block, size);

            // Gauge widget with remaining time label
            let gauge = Gauge::default()
                .block(Block::default().title("Progress").borders(Borders::ALL))
                .gauge_style(Style::default().fg(Color::Green).bg(Color::Black))
                .percent(percentage as u16)
                .label(format!("Time left: {:.0}s", remaining));
            f.render_widget(gauge, size);
        })
            .expect("Failed to draw UI");

        // Exit if countdown is finished or user presses 'q'
        if remaining <= 0.0 {
            break;
        }

        if event::poll(Duration::from_millis(200)).expect("Failed to poll events") {
            if let Event::Key(key) = event::read().expect("Failed to read event") {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }

        // Sleep for a short interval to reduce CPU usage
        thread::sleep(Duration::from_millis(100));
    }

    // --- Clean up terminal -----------------------------------------------
    disable_raw_mode().expect("Failed to disable raw mode");
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    ).expect("Failed to leave alternate screen");
    terminal.show_cursor().expect("Failed to show cursor");
    // -----------------------------------------------------------------------

    // Do endzeit action
    if let Some(exec_command) = args.execute {
        match execute_file(&exec_command) {
            Ok(_) => println!("File executed successfully"),
            Err(e) => eprintln!("Failed to execute file: {}", e),
        }
    } else {
        println!("\nEndzeit reached!");
    }
}
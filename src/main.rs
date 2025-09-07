use clap::Parser;
use chrono::{
    Local,
    NaiveDate,
    NaiveDateTime
};
use std::str::FromStr;
use std::thread;
use std::time::Duration;
use std::process::Command;
use std::io;
use color_eyre::Result;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{
        self,
        Event,
        KeyCode
    },
    layout::{
        Rect
    },
    style::{
        Color,
        Style
    },
    widgets::{
        Gauge,
        // Block,
        // Borders,
        Widget
    },
    DefaultTerminal,
};

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

struct App {
    // target_datetime: NaiveDateTime,
    start_instant: std::time::Instant,
    total_seconds: f64,
    execute_command: Option<String>,
}

fn main() -> Result<()> {
    color_eyre::install()?;
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

    let terminal = ratatui::init();
    let app_result = App::new(target_datetime, args.execute).run(terminal);
    ratatui::restore();
    app_result
}

impl App {
    fn new(target_datetime: NaiveDateTime, execute_command: Option<String>) -> Self {
        let now = Local::now().naive_local();
        let total_duration = target_datetime - now;

        Self {
            // target_datetime,
            start_instant: std::time::Instant::now(),
            total_seconds: total_duration.num_seconds() as f64,
            execute_command,
        }
    }

    fn run(self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;

            if self.is_finished() {
                break;
            }

            if self.should_quit()? {
                break;
            }

            thread::sleep(Duration::from_millis(100));
        }

        self.handle_completion();
        Ok(())
    }

    fn is_finished(&self) -> bool {
        let elapsed = self.start_instant.elapsed().as_secs_f64();
        elapsed >= self.total_seconds
    }

    fn should_quit(&self) -> Result<bool> {
        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                return Ok(key.code == KeyCode::Char('q'));
            }
        }
        Ok(false)
    }

    fn get_remaining_time(&self) -> f64 {
        let elapsed = self.start_instant.elapsed().as_secs_f64();
        (self.total_seconds - elapsed).max(0.0)
    }

    fn get_progress_percentage(&self) -> f64 {
        let elapsed = self.start_instant.elapsed().as_secs_f64();
        if self.total_seconds > 0.0 {
            (elapsed / self.total_seconds).min(1.0) * 100.0
        } else {
            100.0
        }
    }

    fn handle_completion(&self) {
        if let Some(exec_command) = &self.execute_command {
            match execute_file(exec_command) {
                Ok(_) => {},
                Err(e) => eprintln!("Failed to execute file: {}", e),
            }
        }
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_gauge(area, buf);
    }
}

impl App {
    fn render_gauge(&self, area: Rect, buf: &mut Buffer) {
        let remaining = self.get_remaining_time();
        let percentage = self.get_progress_percentage();

        let gauge = Gauge::default()
            // .block(Block::default()
            //     .title(format!("Endzeit: {}", self.target_datetime))  // Use it here
            //     .borders(Borders::ALL))
            .gauge_style(Style::default().fg(Color::Green).bg(Color::Black))
            .percent(percentage as u16)
            .label(format!("Time left: {:.0}s", remaining));

        gauge.render(area, buf);
    }
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
    if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", command_with_args])
            .status()?;
    } else {
        Command::new("sh")
            .args(["-c", command_with_args])
            .status()?;
    };

    Ok(())
}
# endzeit.rust
endzeit in Rust

## Description
This project is a countdown timer written in Rust that can be used to track time until a specified date and time. It provides a command-line interface to input the target date, time, and an optional command to execute upon completion.

## Usage

### Command Line Arguments
- `-d, --date`: The target date in the format `YYYY-MM-DD` (optional, defaults to today).
- `-t, --time`: The target time in the format `HH:MM:SS` (optional, defaults to current time if not provided).
- `--execute`: The command to execute when the countdown reaches zero.

### Example Commands
1. **Basic Countdown**:
   ```sh
   cargo run
   ```
   This will start a countdown from the current date and time.

2. **Countdown with Specific Date and Time**:
   ```sh
   cargo run -- -d 2025-12-31 -t 23:59:00
   ```
   This will start a countdown to December 31, 2025 at 23:59:00.

3. **Countdown with Execution Command**:
   ```sh
   cargo run -- --execute "echo Countdown finished!"
   ```
   This will execute the command `echo Countdown finished!` when the countdown reaches zero.

## Dependencies
The project uses the following Rust dependencies:
- `clap`: For parsing command-line arguments.
- `chrono`: For handling date and time.
- `crossterm`: For terminal-based user interface components.
- `ratatui`: For building text-based interfaces in the terminal.

## How to Build and Run

1. **Clone the Repository**:
   ```sh
   git clone https://github.com/dahead/endzeit.rust.git
   cd endzeit.rust
   ```

2. **Build the Project**:
   ```sh
   cargo build --release
   ```

3. **Run the Executable**:
   ```sh
   cargo run
   ```
   or use the built executable:
   ```sh
   ./target/release/endzeit
   ```

## Notes
- The countdown timer will continue running in the terminal and can be quit by pressing `q`.
- Ensure that any command specified with `--execute` is valid for your operating system.
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Error, Write};
use std::path::Path;

use chrono::Utc;
use colored::Colorize;
use log::{Level, Log, Metadata, Record};
use log::{LevelFilter, SetLoggerError};
use once_cell::sync::Lazy;
use uuid::Uuid;

/// A lazy static instance of `SimpleLogger`.
///
/// The `LOGGER` constant is lazily initialized using the `Lazy` type from the [`lazy_static`](https://docs.rs/lazy_static) crate.
/// It holds an instance of `SimpleLogger` which is used for logging in the application.
///
/// # Note
/// This constant should be used to access the logger instance when logging is needed.
///
static LOGGER: Lazy<SimpleLogger> = Lazy::new(SimpleLogger::new);

/// Initializes the logger with the specified log level filter.
///
/// # Arguments
///
/// * `log` - A string representing the desired log level. Possible values are "info",
///           "warn", "error", "debug", and "trace".
///
/// # Returns
///
/// A `Result` indicating success or failure. If successful, the logger is initialized
/// and the log level filter is set. If an error occurs, the corresponding error is
pub fn init_logger(log: &str) -> Result<(), SetLoggerError> {
    let level_filter = match log {
        "info" => LevelFilter::Info,
        "warn" => LevelFilter::Warn,
        "error" => LevelFilter::Error,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Info,
    };
    log::set_logger(&*LOGGER).map(|()| log::set_max_level(level_filter))
}

/// Struct representing a simple logger.
///
/// The `SimpleLogger` struct is used to log messages.
struct SimpleLogger {
    uuid: Uuid,
}

impl SimpleLogger {
    pub fn new() -> Self {
        let uuid_generated = Uuid::new_v4();
        let _ = File::create(format!("./logs/log_{}.log", uuid_generated)).is_ok();
        SimpleLogger {
            uuid: uuid_generated,
        }
    }
}

impl Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Warn || metadata.target().starts_with("kasuki")
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let text = match record.level() {
                Level::Error => format!("{} : {} - {}", Utc::now(), record.level(), record.args())
                    .truecolor(230, 6, 6),
                Level::Warn => format!("{} : {} - {}", Utc::now(), record.level(), record.args())
                    .truecolor(230, 84, 6),
                Level::Info => format!("{} : {} - {}", Utc::now(), record.level(), record.args())
                    .truecolor(22, 255, 239),
                Level::Debug => format!("{} : {} - {}", Utc::now(), record.level(), record.args())
                    .truecolor(106, 255, 0),
                Level::Trace => format!("{} : {} - {}", Utc::now(), record.level(), record.args())
                    .truecolor(255, 0, 204),
            };

            println!("{}", text);

            let mut file = OpenOptions::new()
                .write(true)
                .append(true)
                .open(format!("./logs/log_{}.log", &self.uuid))
                .unwrap();

            writeln!(file, "{}", text).unwrap();
        }
    }

    fn flush(&self) {}
}

/// Removes old logs from the "./logs" directory.
///
/// # Errors
///
/// This function will return an error if:
///
/// - The directory "./logs" does not exist.
/// - There is an error reading the directory.
/// - There is an error getting the metadata of a file.
/// - There is an error removing a file.
pub fn remove_old_logs() -> Result<(), Error> {
    let path = Path::new("./logs");
    let mut entries: Vec<_> = fs::read_dir(path)?.filter_map(Result::ok).collect();

    // Sort the entries by modification time
    entries.sort_by_key(|e| e.metadata().unwrap().modified().unwrap());

    // Remove the oldest ones until there are only 5 left
    for entry in entries.iter().clone().take(entries.len().saturating_sub(5)) {
        fs::remove_file(entry.path())?;
    }

    Ok(())
}

/// Creates a directory named "logs" if it doesn't exist.
/// # Errors
///
/// Returns an `std::io::Result` indicating whether the directory was successfully created or encountered an error.
///
/// # Remarks
///
/// If the directory already exists, this function does nothing and returns `Ok(())`.
pub fn create_log_directory() -> std::io::Result<()> {
    fs::create_dir_all("./logs")
}

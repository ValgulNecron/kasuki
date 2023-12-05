use crate::error_enum::AppError;
use crate::error_enum::AppError::SetLoggerError;
use chrono::Utc;
use colored::{Color, Colorize};
use once_cell::sync::Lazy;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Error, Write};
use std::path::Path;
use tracing::{Event, Level, Subscriber};
use tracing_core::*;
use tracing_subscriber::layer::{Context, SubscriberExt};
use tracing_subscriber::Layer;
use uuid::Uuid;

/// A lazy static instance of `SimpleLogger`.
///
/// The `LOGGER` constant is lazily initialized using the `Lazy` type from the [`lazy_static`](https://docs.rs/lazy_static) crate.
/// It holds an instance of `SimpleLogger` which is used for logging in the application.
///
/// # Note
/// This constant should be used to access the logger instance when logging is needed.
///
static LOGGER: Lazy<SimpleSubscriber> = Lazy::new(SimpleSubscriber::new);

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

pub fn init_logger(log: &str) -> Result<(), AppError> {
    let level_filter = match log {
        "info" => LevelFilter::INFO,
        "warn" => LevelFilter::WARN,
        "error" => LevelFilter::ERROR,
        "debug" => LevelFilter::DEBUG,
        "trace" => LevelFilter::TRACE,
        _ => LevelFilter::INFO,
    };

    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(level_filter)
        .finish()
        .with(SimpleSubscriber::new());
    tracing::subscriber::set_global_default(subscriber)
        .map_err(|_| SetLoggerError(String::from("Error creating the Logger")))?;

    Ok(())
}

/// Struct representing a simple logger.
///
/// The `SimpleLogger` struct is used to log messages.
struct SimpleSubscriber {
    uuid: Uuid,
}

impl SimpleSubscriber {
    pub fn new() -> Self {
        let uuid_generated = Uuid::new_v4();
        let _ = File::create(format!("./logs/log_{}.log", uuid_generated)).is_ok();
        SimpleSubscriber {
            uuid: uuid_generated,
        }
    }
}

impl<S: Subscriber> Layer<S> for SimpleSubscriber {
    fn on_event(&self, event: &Event, ctx: Context<S>) {
        let level = event.metadata().level();
        let color = match level {
            &Level::ERROR => RgbColor {
                red: 230,
                green: 6,
                blue: 6,
            },
            &Level::WARN => RgbColor {
                red: 230,
                green: 84,
                blue: 6,
            },
            &Level::INFO => RgbColor {
                red: 22,
                green: 255,
                blue: 239,
            },
            &Level::DEBUG => RgbColor {
                red: 106,
                green: 255,
                blue: 0,
            },
            &Level::TRACE => RgbColor {
                red: 255,
                green: 0,
                blue: 204,
            },
        };

        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(format!("./logs/log_{}.log", &self.uuid))
            .unwrap();
        let level_str =
            level
                .to_string()
                .truecolor(color.get_red(), color.get_green(), color.get_blue());
        let target = event.metadata().target();
        let target_str = target.truecolor(color.get_red(), color.get_green(), color.get_blue());
        let mut visitor = MessageVisitor::new();
        event.record(&mut visitor);
        let message = visitor.message;
        let date = Utc::now().to_string().color(Color::Black);
        let message = message.color(Color::BrightBlack);

        let text = format!("{} - {} / {} {}", date, level_str, target_str, message);
        println!("{}", text);

        writeln!(file, "{}", text).unwrap();
    }
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

struct MessageVisitor {
    message: String,
}

impl MessageVisitor {
    fn new() -> Self {
        MessageVisitor {
            message: String::new(),
        }
    }
}

impl field::Visit for MessageVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
        }
    }
}

pub struct RgbColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl RgbColor {
    pub fn new(red: u8, green: u8, blue: u8) -> RgbColor {
        RgbColor { red, green, blue }
    }

    pub fn set_red(&mut self, red: u8) {
        self.red = red;
    }

    pub fn set_green(&mut self, green: u8) {
        self.green = green;
    }

    pub fn set_blue(&mut self, blue: u8) {
        self.blue = blue;
    }

    pub fn get_red(&self) -> u8 {
        self.red
    }

    pub fn get_green(&self) -> u8 {
        self.green
    }

    pub fn get_blue(&self) -> u8 {
        self.blue
    }
}

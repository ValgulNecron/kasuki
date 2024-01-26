use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Error;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;

use chrono::Utc;
use tracing_core::*;
use tracing_subscriber::filter::{Directive, EnvFilter};
use tracing_subscriber::layer::{Context, SubscriberExt};
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, Layer};
use uuid::Uuid;

use crate::constant::OTHER_CRATE_LEVEL;
use crate::error_enum::AppError;
use crate::error_enum::AppError::SetLoggerError;

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
    let kasuki_filter = match log {
        "warn" => "kasuki=warn",
        "error" => "kasuki=error",
        "debug" => "kasuki=debug",
        "trace" => "kasuki=trace",
        _ => "kasuki=info",
    };

    let crate_log = get_directive(OTHER_CRATE_LEVEL)?; // "warn" by default (see constant.rs
    let kasuki_log = get_directive(kasuki_filter)?;

    let filter = EnvFilter::from_default_env()
        .add_directive(crate_log)
        .add_directive(kasuki_log);

    let format = fmt::layer().with_ansi(true);

    tracing_subscriber::registry()
        .with(filter)
        .with(format)
        .with(SimpleSubscriber::new())
        .init();

    Ok(())
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
        fs::remove_file(entry.path())?
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
    fn on_event(&self, event: &Event, _ctx: Context<S>) {
        let level = event.metadata().level();
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(format!("./logs/log_{}.log", &self.uuid))
            .unwrap();
        let level_str = level.to_string();
        let target = event.metadata().target();
        let target_str = target;
        let mut visitor = MessageVisitor::new();
        event.record(&mut visitor);
        let message = visitor.message;
        let date = Utc::now().to_string();

        let text = format!("{} - {} | {} - {}", date, level_str, target_str, message);

        writeln!(file, "{}", text).unwrap();
    }
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
            self.message = format!("{:?}", value)
        }
    }
}

fn get_directive(filter: &str) -> Result<Directive, AppError> {
    let directive = match Directive::from_str(filter) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{}", e);
            return Err(SetLoggerError(String::from("Error creating the Logger")));
        }
    };
    Ok(directive)
}

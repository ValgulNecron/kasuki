use std::fs;
use std::io::Error;
use std::path::Path;
use std::str::FromStr;

use tracing_subscriber::filter::{Directive, EnvFilter};
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;

use crate::constant::{GUARD, LOGS_PATH, LOGS_PREFIX, MAX_LOG_RETENTION_DAYS, OTHER_CRATE_LEVEL};
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

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

    let file_appender = tracing_appender::rolling::daily(LOGS_PATH, LOGS_PREFIX);
    let (file_appender_non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    unsafe {
        GUARD = Some(guard);
    }

    let format = fmt::layer().with_ansi(true);

    let registry = tracing_subscriber::registry()
        .with(filter)
        .with(format)
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(file_appender_non_blocking)
                .with_ansi(false),
        );

    tracing::subscriber::set_global_default(registry).map_err(|e| {
        AppError::new(
            format!("Error creating the Logger. {}", e),
            ErrorType::Logging,
            ErrorResponseType::None,
        )
    })?;

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
    unsafe {
        for entry in entries.iter().clone().take(
            entries
                .len()
                .saturating_sub(MAX_LOG_RETENTION_DAYS as usize),
        ) {
            fs::remove_file(entry.path())?
        }
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

fn get_directive(filter: &str) -> Result<Directive, AppError> {
    let directive = match Directive::from_str(filter) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{}", e);
            return Err(AppError::new(
                format!(
                    "Error creating the Logger. Please check the log level filter. {}",
                    e
                ),
                ErrorType::Logging,
                ErrorResponseType::None,
            ));
        }
    };
    Ok(directive)
}

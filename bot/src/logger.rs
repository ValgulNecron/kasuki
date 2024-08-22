use std::error::Error;
use std::fs;
use std::str::FromStr;
use tracing_appender::rolling::Rotation;
use tracing_subscriber::filter::{Directive, EnvFilter};
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;

use crate::constant::{GUARD, LOGS_PATH, LOGS_PREFIX, LOGS_SUFFIX, OTHER_CRATE_LEVEL};

/// Initializes the logger for the application.
///
/// This function sets up the logger based on the provided log level. It creates a filter for the log level,
/// sets up a file appender for writing logs to a file, and sets the global default subscriber for the application.
///
/// # Arguments
///
/// * `log` - A string slice that holds the log level. It can be "warn", "error", "debug", "trace", or any other value (which defaults to "info").
///
/// # Returns
///
/// * `Result<(), AppError>` - On success, the function returns `Ok(())`.
///   If the function fails to initialize the logger, it returns `Err(AppError)`.
///
/// # Errors
///
/// This function will return an error if there's a problem creating the directives for the log levels,
/// building the file appender, or setting the global default subscriber.
pub fn init_logger(log: &str, max_log_retention_days: u32) -> Result<(), Box<dyn Error>> {
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

    let log_prefix = LOGS_PREFIX;
    let log_suffix = LOGS_SUFFIX;
    let logs_path = LOGS_PATH;

    let file_appender = tracing_appender::rolling::Builder::new()
        .filename_prefix(log_prefix)
        .filename_suffix(log_suffix)
        .rotation(Rotation::DAILY)
        .max_log_files(max_log_retention_days as usize)
        .build(logs_path)?;
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

    tracing::subscriber::set_global_default(registry)?;

    Ok(())
}

/// This function creates a directory named "logs" in the current directory.
///
/// # Returns
///
/// * `std::io::Result<()>` - On success, the function returns `Ok(())`.
///   If the function fails to create the directory, it returns `Err`
/// containing the details of the failure.
///
/// # Errors
///
/// This function will return an error if the `fs::create_dir_all`
/// function fails to create the directory.
pub fn create_log_directory() -> std::io::Result<()> {
    fs::create_dir_all("../logs")
}

/// This function attempts to create a `Directive` from a given filter string.
///
/// # Arguments
///
/// * `Filter` - A string slice that holds the filter value.
///
/// # Returns
///
/// * `Result<Directive, AppError>` - On success, the function returns `Ok(Directive)`.
///   If the function fails to create a `Directive` from the string, it returns `Err(AppError)`.
///
/// # Errors
///
/// This function will return an error if the `Directive::from_str`
/// fails to create a `Directive` from the filter string.
/// The error is of type `AppError` with a message indicating the failure reason,
/// an `ErrorType::Logging`, and an `ErrorResponseType::None`.
fn get_directive(filter: &str) -> Result<Directive, Box<dyn Error>> {
    let directive = Directive::from_str(filter)?;

    Ok(directive)
}

use std::fs;
use std::str::FromStr;

use tracing_appender::rolling::Rotation;
use tracing_subscriber::filter::{Directive, EnvFilter};
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;

use crate::constant::{
    GUARD, LOGS_PATH, LOGS_PREFIX, LOGS_SUFFIX, MAX_LOG_RETENTION_DAYS, OTHER_CRATE_LEVEL,
};
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

/// Initializes the logger for the application.
///
/// # Arguments
///
/// * `log` - A string slice that holds the log level for the kasuki crate.
///
/// # Description
///
/// The function first creates a filter string for the kasuki crate based on the `log` argument.
/// It then gets the directive for the other crates and the kasuki crate.
/// The function sets up a daily rolling file appender and a non-blocking writer for the logs.
/// It also stores the guard for the non-blocking writer in a global static variable.
/// Finally,
/// it sets up the global default subscriber for the tracing crate with the created filter and format layer.
///
/// # Returns
///
/// * `Result<(), AppError>` - On success, the function returns `Ok(())`.
///   If the function fails at any point
/// (e.g., getting the directive, setting the global default subscriber),
/// it returns `Err(AppError)`
///   containing the details of the failure.
///
/// # Errors
///
/// This function will return an error if the `get_directive`
/// function or the `tracing::subscriber::set_global_default`
/// function fails.
/// The error is of type `AppError` with a message indicating the failure reason,
/// an `ErrorType::Logging`, and an `ErrorResponseType::None`.
///
/// # Example
///
/// ```
/// let result = init_logger("warn");
/// assert!(result.is_ok());
/// ```
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

    let file_appender = tracing_appender::rolling::Builder::new()
        .filename_prefix(LOGS_PREFIX)
        .filename_suffix(LOGS_SUFFIX)
        .rotation(Rotation::DAILY)
        .max_log_files(*MAX_LOG_RETENTION_DAYS as usize)
        .build(LOGS_PATH)
        .unwrap();
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
///
/// # Example
///
/// ```
/// let result = create_log_directory();
/// assert!(result.is_ok());
/// ```
pub fn create_log_directory() -> std::io::Result<()> {
    fs::create_dir_all("./logs")
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
///
/// # Example
///
/// ```
/// let directive = get_directive("warn");
/// assert!(directive.is_ok());
/// ```
fn get_directive(filter: &str) -> Result<Directive, AppError> {
    Directive::from_str(filter).map_err(|e| {
        eprintln!("{}", e);
        AppError::new(
            format!(
                "Error creating the Logger. Please check the log level filter. {}",
                e
            ),
            ErrorType::Logging,
            ErrorResponseType::None,
        )
    })
}

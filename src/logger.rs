use std::fs;
use std::str::FromStr;

use tracing_appender::rolling::Rotation;
use tracing_subscriber::filter::{Directive, EnvFilter};
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;

use crate::constant::{
    APP_TUI, GUARD, LOGS_PATH, LOGS_PREFIX, LOGS_SUFFIX, MAX_LOG_RETENTION_DAYS, OTHER_CRATE_LEVEL,
};
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

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

    if *APP_TUI {
        let registry = tracing_subscriber::registry().with(filter).with(
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
    } else {
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
    }

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

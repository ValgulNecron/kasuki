//! Initializes a logger with specific configurations for log retention, log levels, and file output.
use std::fs;
use std::str::FromStr;
use tracing_appender::rolling::Rotation;
use tracing_subscriber::filter::{Directive, EnvFilter};
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;

use crate::constant::{LOGS_PATH, LOGS_PREFIX, LOGS_SUFFIX, OTHER_CRATE_LEVEL};
use anyhow::{Context, Result};
use tracing_appender::non_blocking::WorkerGuard;

/// Initializes a logger with the specified configuration.
///
/// This function sets up a logging framework using the `tracing` crate. It configures
/// the log level, log output paths, retention policies, and other relevant parameters.
///
/// # Parameters
/// - `log`: A string slice representing the desired log level. Supported values are:
///   - `"warn"`: Logs warnings and higher severity levels.
///   - `"error"`: Logs errors only.
///   - `"debug"`: Logs debug information and higher severity levels.
///   - `"trace"`: Logs detailed tracing information and higher severity levels.
///   - Any other value will default to the `"info"` log level.
/// - `max_log_retention_days`: The maximum number of days to retain log files. Log files older
///   than this value will be automatically deleted. Must be a non-negative integer.
///
/// # Returns
/// - `Ok(WorkerGuard)`: Returns a `WorkerGuard` object that must be held to ensure
///   all logs are properly flushed before the application exits.
/// - `Err(Error)`: Returns an error if the logger could not be initialized due to
///   problems such as file creation or setting up the global tracing subscriber.
///
/// # Behavior
/// - This function sets up directives for filtering logs from the `kasuki` module and other modules.
/// - Logs are output to files located in a directory specified by the `LOGS_PATH` constant.
/// - File rotation is configured to be daily, and the filenames include custom prefixes and suffixes
///   defined by `LOGS_PREFIX` and `LOGS_SUFFIX` constants.
/// - Logs older than the specified retention period are deleted automatically.
///
/// # Errors
/// This function may return an error in the following cases:
/// - If the environment variable for setting log levels cannot be parsed.
/// - If the log file appender cannot be created.
/// - If setting the global default subscriber fails.
///
/// # Example
/// ```
/// use std::error::Error;
///
/// fn main() -> Result<(), Box<dyn Error>> {
///     let log_level = "debug"; // Set desired log level
///     let retention_days = 7;  // Retain logs for 7 days
///
///     let guard = init_logger(log_level, retention_days)?;
///
///     // Application code here
///     
///     drop(guard); // Ensure logs are properly flushed
///     Ok(())
/// }
/// ```
///
/// # Notes
/// - The `WorkerGuard` returned by this function should be retained and dropped explicitly at
///   the end of your application. Dropping it ensures that all logs are flushed before
///   terminating the program.
/// - This function depends on several constants (`LOGS_PREFIX`, `LOGS_SUFFIX`, `LOGS_PATH`)
///   which should be predefined in the application's configuration.
///
/// # Dependencies
/// - Requires the `tracing`, `tracing-appender`, and `tracing-subscriber` crates.
/// - Requires the `anyhow` crate for error handling and context propagation.
pub fn init_logger(log: &str, max_log_retention_days: u32) -> Result<WorkerGuard> {
	let kasuki_filter = match log {
		"warn" => "kasuki=warn",
		"error" => "kasuki=error",
		"debug" => "kasuki=debug",
		"trace" => "kasuki=trace",
		_ => "kasuki=info",
	};

	let crate_log = get_directive(OTHER_CRATE_LEVEL)?;

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
		.build(logs_path)
		.context("Failed to create file appender")?;

	let (file_appender_non_blocking, guard) = tracing_appender::non_blocking(file_appender);

	let format = fmt::layer().with_ansi(true);

	let registry = tracing_subscriber::registry()
		.with(filter)
		.with(format)
		.with(
			tracing_subscriber::fmt::layer()
				.with_writer(file_appender_non_blocking)
				.with_ansi(false),
		);

	tracing::subscriber::set_global_default(registry)
		.context("Failed to set global default subscriber")?;

	Ok(guard)
}

/// Creates a log directory if it does not already exist.
///
/// This function attempts to create a directory named `logs`
/// in the parent directory of the current working directory.
/// If the directory already exists, no error is returned.
/// Any errors encountered while attempting to create the directory
/// will be returned to the caller wrapped in a `Result` with a
/// descriptive context message.
///
/// # Returns
///
/// * `Ok(())` - If the directory is successfully created or already exists.
/// * `Err(Error)` - If there is an issue creating the directory.
///
/// # Errors
///
/// Returns an error if the process lacks permissions to create the directory,
/// if the parent directory is not writable, or if an unexpected filesystem
/// error occurs during the creation of the directory.
///
/// # Examples
///
/// ```
/// use your_crate::create_log_directory;
///
/// if let Err(e) = create_log_directory() {
///     eprintln!("Error: {}", e);
/// } else {
///     println!("Log directory created or already exists.");
/// }
/// ```
pub fn create_log_directory() -> Result<()> {
	fs::create_dir_all("../logs").context("Failed to create log directory")
}

/// Attempts to create a `Directive` from a given filter string.
///
/// # Arguments
///
/// * `filter` - A string slice representing the directive filter to parse.
///
/// # Returns
///
/// * `Ok(Directive)` if the string is successfully parsed into a `Directive`.
/// * `Err(anyhow::Error)` if parsing the filter string into a `Directive` fails,
///   with an attached context describing the failure.
///
/// # Errors
///
/// This function returns an error if the `Directive::from_str` method fails to parse
/// the provided filter string.
///
/// # Examples
///
/// ```
/// let filter = "info";
/// match get_directive(filter) {
///     Ok(directive) => println!("Successfully created directive: {:?}", directive),
///     Err(e) => eprintln!("Error encountered: {}", e),
/// }
/// ```
fn get_directive(filter: &str) -> Result<Directive> {
	Directive::from_str(filter).context("Failed to create directive")
}

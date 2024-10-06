use std::fs;
use std::str::FromStr;
use tracing_appender::rolling::Rotation;
use tracing_subscriber::filter::{Directive, EnvFilter};
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;

use crate::constant::{LOGS_PATH, LOGS_PREFIX, LOGS_SUFFIX, OTHER_CRATE_LEVEL};
use anyhow::{Context, Result};
use tracing_appender::non_blocking::WorkerGuard;

pub fn init_logger(log: &str, max_log_retention_days: u32) -> Result<WorkerGuard> {
    let kasuki_filter = match log {
        "warn" => "kasuki=warn",
        "error" => "kasuki=error",
        "debug" => "kasuki=debug",
        "trace" => "kasuki=trace",
        _ => "kasuki=info",
    };

    let crate_log = get_directive(OTHER_CRATE_LEVEL).context("Failed to get crate directive")?;

    let kasuki_log = get_directive(kasuki_filter).context("Failed to get kasuki directive")?;

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

pub fn create_log_directory() -> Result<()> {
    fs::create_dir_all("../logs").context("Failed to create log directory")
}

fn get_directive(filter: &str) -> Result<Directive> {
    Directive::from_str(filter).context("Failed to create directive")
}

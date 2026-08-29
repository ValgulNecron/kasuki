//! Disk-aware log rotation, managed by the process that writes the logs.
//!
//! `tracing_appender`'s daily rotation caps the *number* of retained files, not
//! their total size — a spike in log volume can still fill the disk, after which
//! the appender can no longer open today's file and the process fails to start.
//! This task polls free space on the log volume and, when it drops below a
//! configured threshold, deletes this process's own oldest log files (always
//! keeping the newest, active one) until enough space is reclaimed.
//!
//! The core routine is also run once synchronously at startup, before the logger
//! is initialised, so a process coming up to an already-full volume can free
//! space and boot instead of crash-looping.

use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::time::Duration;

use shared::config::LoggingConfig;
use tracing::{error, info};

use crate::constant::{LOGS_PATH, LOGS_PREFIX, LOGS_SUFFIX};

/// Returns `(available_bytes, total_bytes)` for the filesystem containing `path`.
fn filesystem_space(path: &Path) -> std::io::Result<(u64, u64)> {
	let c_path = CString::new(path.as_os_str().as_bytes())
		.map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

	// SAFETY: `c_path` is a valid NUL-terminated path and `stat` is zeroed then
	// fully written by `statvfs`; we only read it on success (rc == 0).
	let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
	let rc = unsafe { libc::statvfs(c_path.as_ptr(), &mut stat) };
	if rc != 0 {
		return Err(std::io::Error::last_os_error());
	}

	let frsize = stat.f_frsize as u64;
	let available = (stat.f_bavail as u64).saturating_mul(frsize);
	let total = (stat.f_blocks as u64).saturating_mul(frsize);
	Ok((available, total))
}

/// A candidate log file discovered on disk.
struct LogFile {
	path: std::path::PathBuf,
	mtime: std::time::SystemTime,
	size: u64,
}

/// Chooses which log files to delete to reclaim `needed` bytes: oldest first,
/// and never the newest (currently-written) file. Pure so it can be unit-tested
/// without touching a filesystem.
fn select_logs_to_prune(mut files: Vec<LogFile>, needed: u64) -> Vec<LogFile> {
	if needed == 0 || files.len() <= 1 {
		return Vec::new();
	}

	// Oldest first; the last element (newest) is always retained.
	files.sort_by_key(|f| f.mtime);
	let prunable = files.len() - 1;

	let mut selected = Vec::new();
	let mut reclaimed = 0u64;
	for file in files.into_iter().take(prunable) {
		if reclaimed >= needed {
			break;
		}
		reclaimed += file.size;
		selected.push(file);
	}
	selected
}

/// Deletes oldest log files (matching `prefix`/`suffix`) in `dir` until the
/// filesystem has at least `target_free_bytes` available, always keeping the
/// newest file. Returns `(files_deleted, bytes_freed)`. `report` receives
/// human-readable progress so the caller can route it to stderr (pre-logger) or
/// `tracing` (once the logger is up).
fn prune_logs_until_free(
	dir: &Path, prefix: &str, suffix: &str, target_free_bytes: u64, report: &dyn Fn(&str),
) -> std::io::Result<(usize, u64)> {
	let (available, _total) = filesystem_space(dir)?;
	let needed = target_free_bytes.saturating_sub(available);
	if needed == 0 {
		return Ok((0, 0));
	}

	let mut files: Vec<LogFile> = Vec::new();
	for entry in std::fs::read_dir(dir)? {
		let entry = match entry {
			Ok(e) => e,
			Err(_) => continue,
		};
		let name = entry.file_name();
		let name = name.to_string_lossy();
		if !name.starts_with(prefix) || !name.ends_with(suffix) {
			continue;
		}
		let meta = match entry.metadata() {
			Ok(m) => m,
			Err(_) => continue,
		};
		if !meta.is_file() {
			continue;
		}
		let mtime = meta.modified().unwrap_or(std::time::UNIX_EPOCH);
		files.push(LogFile {
			path: entry.path(),
			mtime,
			size: meta.len(),
		});
	}

	let mut deleted = 0usize;
	let mut freed = 0u64;
	for file in select_logs_to_prune(files, needed) {
		match std::fs::remove_file(&file.path) {
			Ok(()) => {
				deleted += 1;
				freed += file.size;
				report(&format!(
					"log cleanup: removed {} (~{} KiB)",
					file.path.display(),
					file.size / 1024
				));
			},
			Err(e) => {
				report(&format!(
					"log cleanup: failed to remove {}: {}",
					file.path.display(),
					e
				));
			},
		}
	}

	if deleted > 0 {
		report(&format!(
			"log cleanup: removed {} file(s), freed ~{} MiB",
			deleted,
			freed / (1024 * 1024)
		));
	}
	Ok((deleted, freed))
}

/// Checks the log volume and prunes oldest logs if free space is below
/// `disk_min_free_percent`, down to `disk_target_free_percent`.
fn enforce_log_budget(
	logging: &LoggingConfig, report: &dyn Fn(&str),
) -> std::io::Result<(usize, u64)> {
	let dir = Path::new(LOGS_PATH);
	// Make sure the directory exists so both statvfs and the appender have a
	// valid target even on a first-ever run.
	let _ = std::fs::create_dir_all(dir);

	let (available, total) = filesystem_space(dir)?;
	if total == 0 {
		return Ok((0, 0));
	}

	let min_free = total / 100 * u64::from(logging.disk_min_free_percent);
	if available >= min_free {
		return Ok((0, 0));
	}

	let target_free = total / 100 * u64::from(logging.disk_target_free_percent);
	report(&format!(
		"log volume low: {} MiB free of {} MiB (< {}%); pruning oldest logs toward {}% free",
		available / (1024 * 1024),
		total / (1024 * 1024),
		logging.disk_min_free_percent,
		logging.disk_target_free_percent,
	));
	prune_logs_until_free(dir, LOGS_PREFIX, LOGS_SUFFIX, target_free, report)
}

/// One-shot cleanup run before the logger exists; reports to stderr. Safe to
/// call unconditionally — it no-ops when there is enough free space.
pub fn enforce_log_budget_at_startup(logging: &LoggingConfig) {
	if let Err(e) = enforce_log_budget(logging, &|m| eprintln!("{}", m)) {
		eprintln!("Startup log cleanup failed: {}", e);
	}
}

/// Background task: periodically enforces the log disk budget. Filesystem work
/// runs on a blocking thread so it never stalls the async runtime (NFS calls can
/// block for a while).
#[tracing::instrument(skip(logging), level = "info")]
pub async fn log_cleanup_task(logging: LoggingConfig) {
	let interval = Duration::from_secs(logging.disk_cleanup_interval_secs.max(30));
	info!(
		"Starting log disk cleanup task (interval: {}s, min free: {}%, target free: {}%)",
		interval.as_secs(),
		logging.disk_min_free_percent,
		logging.disk_target_free_percent,
	);

	loop {
		tokio::time::sleep(interval).await;

		let logging = logging.clone();
		let result =
			tokio::task::spawn_blocking(move || enforce_log_budget(&logging, &|m| info!("{}", m)))
				.await;

		match result {
			Ok(Ok(_)) => {},
			Ok(Err(e)) => error!("log cleanup failed: {}", e),
			Err(e) => error!("log cleanup task panicked: {}", e),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::time::{Duration, UNIX_EPOCH};

	fn file(name: &str, secs: u64, size: u64) -> LogFile {
		LogFile {
			path: std::path::PathBuf::from(name),
			mtime: UNIX_EPOCH + Duration::from_secs(secs),
			size,
		}
	}

	#[test]
	fn selects_nothing_when_no_space_needed() {
		let files = vec![file("a", 1, 100), file("b", 2, 100)];
		assert!(select_logs_to_prune(files, 0).is_empty());
	}

	#[test]
	fn never_prunes_the_only_file() {
		let files = vec![file("only", 1, 5000)];
		assert!(select_logs_to_prune(files, 4000).is_empty());
	}

	#[test]
	fn always_keeps_the_newest_file() {
		// Even if we need more than the older files can supply, the newest
		// (highest mtime) is never selected for deletion.
		let files = vec![
			file("old", 1, 100),
			file("mid", 2, 100),
			file("new", 3, 100),
		];
		let selected = select_logs_to_prune(files, 10_000);
		let names: Vec<_> = selected
			.iter()
			.map(|f| f.path.to_string_lossy().to_string())
			.collect();
		assert_eq!(names, vec!["old", "mid"]);
		assert!(!names.contains(&"new".to_string()));
	}

	#[test]
	fn prunes_oldest_first_and_stops_once_enough_reclaimed() {
		let files = vec![
			file("newest", 4, 100),
			file("oldest", 1, 100),
			file("older", 2, 100),
			file("old", 3, 100),
		];
		// Need 150 bytes: deleting the two oldest (100 + 100 = 200) suffices.
		let selected = select_logs_to_prune(files, 150);
		let names: Vec<_> = selected
			.iter()
			.map(|f| f.path.to_string_lossy().to_string())
			.collect();
		assert_eq!(names, vec!["oldest", "older"]);
	}
}

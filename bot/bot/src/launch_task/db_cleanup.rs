use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use shared::config::TaskIntervalConfig;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info};

/// Periodically deletes old `command_usage` and `ping_history` rows
/// to prevent unbounded table growth.
#[tracing::instrument(skip(db_connection, task_intervals), level = "info")]
pub async fn db_cleanup_task(
	db_connection: Arc<DatabaseConnection>, task_intervals: TaskIntervalConfig,
) {
	let interval_hours = task_intervals.db_cleanup_interval_hours;
	let retention_days = task_intervals.db_retention_days;

	info!(
		"Starting DB cleanup task (interval: {}h, retention: {}d)",
		interval_hours, retention_days
	);

	let interval = Duration::from_secs(interval_hours * 3600);

	loop {
		tokio::time::sleep(interval).await;

		let cutoff = chrono::Utc::now().naive_utc() - chrono::Duration::days(retention_days as i64);

		debug!("Running DB cleanup, deleting rows older than {}", cutoff);

		match shared::database::prelude::CommandUsage::delete_many()
			.filter(shared::database::command_usage::Column::UseTime.lt(cutoff))
			.exec(&*db_connection)
			.await
		{
			Ok(result) => {
				info!(
					"DB cleanup: deleted {} command_usage rows",
					result.rows_affected
				);
			},
			Err(e) => {
				error!("DB cleanup: failed to delete command_usage rows: {}", e);
			},
		}

		match shared::database::prelude::PingHistory::delete_many()
			.filter(shared::database::ping_history::Column::Timestamp.lt(cutoff))
			.exec(&*db_connection)
			.await
		{
			Ok(result) => {
				info!(
					"DB cleanup: deleted {} ping_history rows",
					result.rows_affected
				);
			},
			Err(e) => {
				error!("DB cleanup: failed to delete ping_history rows: {}", e);
			},
		}
	}
}

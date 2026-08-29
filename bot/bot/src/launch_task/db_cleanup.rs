use sea_orm::{
	ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter, Statement,
	TransactionTrait,
};
use shared::config::TaskIntervalConfig;
use shared::image_saver::storage::ImageStore;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info};

/// Periodically prunes long-lived state to prevent unbounded disk growth:
/// - `command_usage`, `ping_history` rows older than `db_retention_days`
/// - `vocal` / `message` rows older than `db_retention_days`, with running totals
///   archived to `vocal_summary` / `message_summary` first so XP stays stable
/// - `oauth_token` rows whose `expires_at` has passed
/// - On-disk images older than `image_retention_days`
#[tracing::instrument(skip(db_connection, task_intervals, image_store), level = "info")]
pub async fn db_cleanup_task(
	db_connection: Arc<DatabaseConnection>, task_intervals: TaskIntervalConfig,
	image_store: Arc<dyn ImageStore>,
) {
	let interval_hours = task_intervals.db_cleanup_interval_hours;
	let retention_days = task_intervals.db_retention_days;
	let image_retention_days = task_intervals.image_retention_days;

	info!(
		"Starting DB cleanup task (interval: {}h, db retention: {}d, image retention: {}d)",
		interval_hours, retention_days, image_retention_days
	);

	let interval = Duration::from_secs(interval_hours * 3600);

	loop {
		tokio::time::sleep(interval).await;

		let now = chrono::Utc::now().naive_utc();
		let cutoff = now - chrono::Duration::days(retention_days as i64);

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

		match prune_vocal_with_rollup(&db_connection, cutoff).await {
			Ok((archived, deleted)) => {
				info!(
					"DB cleanup: rolled up {} vocal groups into vocal_summary, deleted {} vocal rows",
					archived, deleted
				);
			},
			Err(e) => {
				error!("DB cleanup: failed to prune vocal rows: {}", e);
			},
		}

		match prune_message_with_rollup(&db_connection, cutoff).await {
			Ok((archived, deleted)) => {
				info!(
					"DB cleanup: rolled up {} message groups into message_summary, deleted {} message rows",
					archived, deleted
				);
			},
			Err(e) => {
				error!("DB cleanup: failed to prune message rows: {}", e);
			},
		}

		match shared::database::prelude::OAuthToken::delete_many()
			.filter(shared::database::oauth_token::Column::ExpiresAt.lt(now))
			.exec(&*db_connection)
			.await
		{
			Ok(result) => {
				info!(
					"DB cleanup: deleted {} expired oauth_token rows",
					result.rows_affected
				);
			},
			Err(e) => {
				error!("DB cleanup: failed to delete oauth_token rows: {}", e);
			},
		}

		let image_max_age = Duration::from_secs(image_retention_days * 24 * 3600);
		match image_store.cleanup_older_than(image_max_age).await {
			Ok(count) => {
				info!("DB cleanup: removed {} stored image files", count);
			},
			Err(e) => {
				error!("DB cleanup: failed to sweep image store: {}", e);
			},
		}
	}
}

/// Aggregate vocal rows older than `cutoff` into `vocal_summary` (so XP totals
/// don't drop when the rows themselves are pruned), then delete the raw rows.
/// Both operations run in one transaction so we never lose-then-fail.
///
/// Returns `(rolled_up_groups, deleted_rows)`.
async fn prune_vocal_with_rollup(
	db: &DatabaseConnection, cutoff: chrono::NaiveDateTime,
) -> Result<(u64, u64), sea_orm::DbErr> {
	let backend = db.get_database_backend();
	let txn = db.begin().await?;

	let rollup_stmt = Statement::from_sql_and_values(
		backend,
		r#"
		INSERT INTO vocal_summary (user_id, channel_id, session_count, duration_total_seconds)
		SELECT user_id, channel_id, COUNT(*)::bigint, COALESCE(SUM(duration), 0)::bigint
		FROM vocal
		WHERE "end" < $1
		GROUP BY user_id, channel_id
		ON CONFLICT (user_id, channel_id) DO UPDATE SET
			session_count = vocal_summary.session_count + EXCLUDED.session_count,
			duration_total_seconds =
				vocal_summary.duration_total_seconds + EXCLUDED.duration_total_seconds
		"#,
		[cutoff.into()],
	);
	let rolled = txn.execute_raw(rollup_stmt).await?.rows_affected();

	let deleted = shared::database::prelude::Vocal::delete_many()
		.filter(shared::database::vocal::Column::End.lt(cutoff))
		.exec(&txn)
		.await?
		.rows_affected;

	txn.commit().await?;
	Ok((rolled, deleted))
}

/// Same idea as `prune_vocal_with_rollup` but for the `message` table: archive
/// per-(user, channel) message counts into `message_summary` before deleting
/// rows older than `cutoff`.
async fn prune_message_with_rollup(
	db: &DatabaseConnection, cutoff: chrono::NaiveDateTime,
) -> Result<(u64, u64), sea_orm::DbErr> {
	let backend = db.get_database_backend();
	let txn = db.begin().await?;

	let rollup_stmt = Statement::from_sql_and_values(
		backend,
		r#"
		INSERT INTO message_summary (user_id, channel_id, message_count)
		SELECT user_id, channel_id, COUNT(*)::bigint
		FROM message
		WHERE created_at < $1
		GROUP BY user_id, channel_id
		ON CONFLICT (user_id, channel_id) DO UPDATE SET
			message_count = message_summary.message_count + EXCLUDED.message_count
		"#,
		[cutoff.into()],
	);
	let rolled = txn.execute_raw(rollup_stmt).await?.rows_affected();

	let deleted = shared::database::prelude::Message::delete_many()
		.filter(shared::database::message::Column::CreatedAt.lt(cutoff))
		.exec(&txn)
		.await?
		.rows_affected;

	txn.commit().await?;
	Ok((rolled, deleted))
}

use crate::event_handler::BotData;
use anyhow::{Context as AnyhowContext, Result};
use sea_orm::ActiveValue::Set;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use serenity::all::{Context as SerenityContext, ShardId};
use serenity::gateway::ShardRunnerInfo;
use shared::config::TaskIntervalConfig;
use shared::database::ping_history::ActiveModel;
use shared::database::prelude::PingHistory;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, trace};

/// Monitors and records the latency (ping) of each Discord gateway shard.
#[tracing::instrument(skip(ctx, db_connection, task_intervals), level = "info")]
pub async fn ping_manager_thread(
	ctx: SerenityContext, db_connection: Arc<DatabaseConnection>,
	task_intervals: TaskIntervalConfig,
) -> Result<()> {
	info!("Launching the ping monitoring thread!");
	debug!(
		"Ping update interval configured for {} seconds",
		task_intervals.ping_update
	);

	trace!("Attempting to retrieve shard manager from context");
	let shard_manager: Arc<RwLock<HashMap<ShardId, Arc<parking_lot::RwLock<ShardRunnerInfo>>>>> =
		ctx.data::<BotData>().shard_manager.clone();

	let mut interval = tokio::time::interval(Duration::from_secs(task_intervals.ping_update));
	debug!(
		"Set up ping check interval timer for {} seconds",
		task_intervals.ping_update
	);

	let mut cycle_count = 0;
	let mut total_shards_processed = 0;
	let mut successful_updates = 0;

	loop {
		interval.tick().await;
		cycle_count += 1;

		let current_time = chrono::Utc::now();
		info!(
			"Starting ping update cycle #{} at {}",
			cycle_count, current_time
		);
		trace!("Ping update cycle started");

		let read = shard_manager.read().await;
		let shard_count = read.len();
		debug!("Processing {} shards in this cycle", shard_count);

		let mut cycle_successful_updates = 0;
		let mut cycle_failed_updates = 0;

		for (shard_id, shard_arc) in read.iter() {
			trace!("Processing shard {}", shard_id);

			let (now, latency) = {
				let shard_info = shard_arc.read();

				let latency = shard_info
					.latency
					.unwrap_or_default()
					.as_millis()
					.to_string();

				trace!("Shard {} current latency: {}ms", shard_id, latency);

				let now = chrono::Utc::now().naive_utc();

				(now, latency)
			};

			trace!(
				"Inserting ping history record for shard {} with latency {}",
				shard_id,
				latency
			);
			let result = PingHistory::insert(ActiveModel {
				shard_id: Set(shard_id.to_string()),
				latency: Set(latency.clone()),
				timestamp: Set(now),
				..Default::default()
			})
			.exec(&*db_connection)
			.await
			.context(format!(
				"Failed to insert ping history for shard {} with latency {}",
				shard_id, latency
			));

			match result {
				Ok(_) => {
					debug!("Updated ping history for shard {}.", shard_id);
					cycle_successful_updates += 1;
					successful_updates += 1;
					total_shards_processed += 1;
				},
				Err(e) => {
					error!(
						"Failed to update ping history for shard {}: {:#}",
						shard_id, e
					);
					cycle_failed_updates += 1;
					total_shards_processed += 1;
				},
			}
		}

		let cycle_end_time = chrono::Utc::now();
		let cycle_duration = cycle_end_time.signed_duration_since(current_time);

		info!(
			"Ping update cycle #{} completed: processed {} shards ({} successful, {} failed) in {:?}",
			cycle_count,
			shard_count,
			cycle_successful_updates,
			cycle_failed_updates,
			cycle_duration
		);

		debug!(
			"Ping monitoring stats: total processed: {}, success rate: {:.2}%",
			total_shards_processed,
			if total_shards_processed > 0 {
				(successful_updates as f64 / total_shards_processed as f64) * 100.0
			} else {
				0.0
			}
		);

		trace!("Ping update cycle completed");
	}

	#[allow(unreachable_code)]
	Ok(())
}

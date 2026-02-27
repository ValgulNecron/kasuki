mod activity;
mod get_anisong_db;
mod update_random_stats;

use anyhow::{Context, Result};
use serenity::all::Token;
use serenity::http::Http;
use shared::cache::CacheInterface;
use shared::config::WorkerConfig;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, broadcast};
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::activity::anime_activity::manage_activity;
use crate::get_anisong_db::get_anisong;
use crate::update_random_stats::update_random_stats_launcher;

#[tokio::main]
async fn main() -> Result<()> {
	let subscriber = FmtSubscriber::builder()
		.with_max_level(Level::INFO)
		.finish();
	tracing::subscriber::set_global_default(subscriber)
		.context("setting default subscriber failed")?;

	info!("Starting Worker...");

	let config = WorkerConfig::new().context("Failed to load config.toml")?;
	info!("Configuration loaded");

	info!("Connecting to database...");
	let connection = Arc::new(
		config
			.db
			.connect()
			.await
			.context("Failed to connect to database")?,
	);
	info!("Database connected");

	let anilist_cache = Arc::new(RwLock::new(CacheInterface::new()));

	let token = Token::from_str(&config.bot.discord_token).context("Invalid Discord token")?;
	let http = Arc::new(Http::new(token));

	let (shutdown_tx, _) = broadcast::channel::<()>(1);
	let task_intervals = config.task_intervals.clone();

	// Spawn Anisong DB Update Task
	let mut shutdown_rx = shutdown_tx.subscribe();
	let db_clone = connection.clone();
	let intervals_clone = task_intervals.clone();
	let anisong_handle = tokio::spawn(async move {
		info!("Launching anisong database update task");
		let mut interval =
			tokio::time::interval(Duration::from_secs(intervals_clone.anisong_update));
		let mut cycle = 0u64;

		loop {
			tokio::select! {
				_ = shutdown_rx.recv() => {
					info!("Anisong task received shutdown signal");
					break;
				}
				_ = interval.tick() => {
					cycle += 1;
					info!("Starting anisong update cycle #{}", cycle);
					match get_anisong(db_clone.clone()).await {
						Ok(count) => info!("Anisong cycle #{} done, processed: {}", cycle, count),
						Err(e) => error!("Anisong cycle #{} failed: {:#}", cycle, e),
					}
				}
			}
		}
	});

	// Spawn Random Stats Update Task
	let shutdown_rx = shutdown_tx.subscribe();
	let cache_clone = anilist_cache.clone();
	let intervals_clone = task_intervals.clone();
	let db_clone = connection.clone();
	let stats_handle = tokio::spawn(async move {
		update_random_stats_launcher(cache_clone, intervals_clone, db_clone, shutdown_rx).await;
	});

	// Spawn Activity Management Task
	let mut shutdown_rx = shutdown_tx.subscribe();
	let http_clone = http.clone();
	let cache_clone = anilist_cache.clone();
	let db_clone = connection.clone();
	let intervals_clone = task_intervals.clone();
	let activity_handle = tokio::spawn(async move {
		info!("Launching activity management task");
		let mut interval =
			tokio::time::interval(Duration::from_secs(intervals_clone.activity_check));

		loop {
			tokio::select! {
				_ = shutdown_rx.recv() => {
					info!("Activity task received shutdown signal");
					break;
				}
				_ = interval.tick() => {
					manage_activity(
						http_clone.clone(),
						cache_clone.clone(),
						db_clone.clone(),
					)
					.await;
				}
			}
		}
	});

	info!("Worker tasks started. Press Ctrl+C to shutdown.");

	match tokio::signal::ctrl_c().await {
		Ok(()) => info!("Shutdown signal received"),
		Err(err) => error!("Unable to listen for shutdown signal: {}", err),
	}

	info!("Sending shutdown signal to all tasks...");
	let _ = shutdown_tx.send(());

	// Give tasks time to finish current work
	let timeout = Duration::from_secs(10);
	let _ = tokio::time::timeout(timeout, async {
		let _ = tokio::join!(anisong_handle, stats_handle, activity_handle);
	})
	.await;

	info!("Worker shut down.");
	Ok(())
}

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
use tokio::sync::broadcast;
use tracing::{error, info, warn, Level};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::activity::anime_activity::manage_activity;
use crate::get_anisong_db::get_anisong;
use crate::update_random_stats::update_random_stats_launcher;

#[tokio::main]
async fn main() -> Result<()> {
	let config = WorkerConfig::new().context("Failed to load config.toml")?;

	let _sentry_guard = config.sentry_url.as_deref().map(|url| {
		let guard = sentry::init((
			url,
			sentry::ClientOptions {
				release: sentry::release_name!(),
				..Default::default()
			},
		));
		println!("Sentry initialized successfully");
		guard
	});

	let sentry_layer = sentry::integrations::tracing::layer();
	tracing_subscriber::registry()
		.with(tracing_subscriber::filter::LevelFilter::from_level(Level::INFO))
		.with(sentry_layer)
		.with(tracing_subscriber::fmt::layer())
		.init();

	info!("Starting Worker...");
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

	let anilist_cache = Arc::new(
		match CacheInterface::from_config(&config.cache).await {
			Ok(c) => {
				info!("AniList cache initialized with {} backend", config.cache.cache_type);
				c
			},
			Err(e) => {
				warn!(
					"Failed to init cache with {} backend, falling back to memory: {}",
					config.cache.cache_type, e
				);
				CacheInterface::new()
			},
		},
	);

	let token = Token::from_str(&config.bot.discord_token).context("Invalid Discord token")?;
	let http = Arc::new(Http::new(token));

	let (shutdown_tx, _) = broadcast::channel::<()>(1);
	let task_intervals = config.task_intervals.clone();

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

	let shutdown_rx = shutdown_tx.subscribe();
	let cache_clone = anilist_cache.clone();
	let intervals_clone = task_intervals.clone();
	let db_clone = connection.clone();
	let stats_handle = tokio::spawn(async move {
		update_random_stats_launcher(cache_clone, intervals_clone, db_clone, shutdown_rx).await;
	});

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

	let timeout = Duration::from_secs(10);
	let _ = tokio::time::timeout(timeout, async {
		let _ = tokio::join!(anisong_handle, stats_handle, activity_handle);
	})
	.await;

	info!("Worker shut down.");
	Ok(())
}

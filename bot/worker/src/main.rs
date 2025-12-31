mod activity;
mod get_anisong_db;
mod update_random_stats;

use anyhow::{Context, Result};
use sea_orm::DatabaseConnection;
use serenity::all::Token;
use serenity::http::Http;
use shared::cache::CacheInterface;
use shared::config::{Config, DbConfig, TaskIntervalConfig};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::activity::anime_activity::manage_activity;
use crate::get_anisong_db::get_anisong;
use crate::update_random_stats::update_random_stats_launcher;

#[tokio::main]
async fn main() -> Result<()> {
	// Initialize logging
	let subscriber = FmtSubscriber::builder()
		.with_max_level(Level::INFO)
		.finish();
	tracing::subscriber::set_global_default(subscriber)
		.context("setting default subscriber failed")?;

	info!("Starting Worker...");

	// Load configuration
	let config = Config::new().context("Failed to load config.toml")?;
	info!("Configuration loaded.");

	// Initialize Database
	let db_url = get_url(config.db.clone());
	info!("Connecting to database...");
	let connection = sea_orm::Database::connect(db_url)
		.await
		.context("Failed to connect to database")?;
	let connection = Arc::new(connection);
	info!("Database connected.");

	// Initialize Cache
	let anilist_cache = Arc::new(RwLock::new(CacheInterface::new()));

	// Initialize Discord HTTP Client
	let token = Token::from_str(&config.bot.discord_token).context("Invalid Discord token")?;
	let http = Arc::new(Http::new(token));

	// Task Intervals
	let task_intervals = config.task_intervals.clone();

	// Spawn Anisong DB Update Task
	let db_clone = connection.clone();
	let intervals_clone = task_intervals.clone();
	tokio::spawn(async move {
		update_anisong_db(db_clone, intervals_clone).await;
	});

	// Spawn Random Stats Update Task
	let cache_clone = anilist_cache.clone();
	let intervals_clone_2 = task_intervals.clone();
	tokio::spawn(async move {
		update_random_stats_launcher(cache_clone, intervals_clone_2).await;
	});

	// Spawn Activity Management Task
	let http_clone = http.clone();
	let cache_clone_2 = anilist_cache.clone();
	let db_clone_2 = connection.clone();
	let intervals_clone_3 = task_intervals.clone();
	tokio::spawn(async move {
		info!("Launching activity management task");
		let mut interval =
			tokio::time::interval(Duration::from_secs(intervals_clone_3.activity_check));
		loop {
			interval.tick().await;
			manage_activity(
				http_clone.clone(),
				cache_clone_2.clone(),
				db_clone_2.clone(),
			)
			.await;
		}
	});

	info!("Worker tasks started. Waiting for signal...");

	// Wait for Ctrl+C
	match tokio::signal::ctrl_c().await {
		Ok(()) => info!("Shutting down worker..."),
		Err(err) => error!("Unable to listen for shutdown signal: {}", err),
	}

	Ok(())
}

async fn update_anisong_db(db: Arc<DatabaseConnection>, task_intervals: TaskIntervalConfig) {
	info!("Launching the anisongdb update background task");
	let mut interval = tokio::time::interval(Duration::from_secs(task_intervals.anisong_update));
	let mut update_count = 0;

	loop {
		interval.tick().await;
		update_count += 1;
		info!("Starting anisong database update cycle #{}", update_count);

		match get_anisong(db.clone()).await {
			Ok(count) => info!(
				"Anisong database update cycle #{} completed. Processed: {}",
				update_count, count
			),
			Err(e) => error!(
				"Anisong database update cycle #{} failed: {:#}",
				update_count, e
			),
		}
	}
}

pub fn get_url(db_config: DbConfig) -> String {
	match db_config.db_type.as_str() {
		"postgresql" => {
			let host = db_config.host.unwrap_or_else(|| "localhost".to_string());
			let port = db_config.port.unwrap_or(5432);
			let user = db_config.user.unwrap_or_else(|| "postgres".to_string());
			let password = db_config.password.unwrap_or_default();
			let db_name = db_config.database.unwrap_or_else(|| "kasuki".to_string());

			let param = vec![("user", user.as_str()), ("password", password.as_str())];
			let param = serde_urlencoded::to_string(&param).unwrap();

			format!("postgresql://{}:{}/{}?{}", host, port, db_name, param)
		},
		_ => {
			panic!("Unsupported database type: {}", db_config.db_type);
		},
	}
}

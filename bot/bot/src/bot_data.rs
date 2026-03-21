use chrono::{DateTime, Timelike, Utc};
use lavalink_rs::client::LavalinkClient;
use reqwest::Client;
use sea_orm::ActiveValue::Set;
use sea_orm::{
	ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
};
use serenity::all::{CurrentApplicationInfo, ShardId};
use serenity::gateway::ShardRunnerInfo;
use shared::cache::CacheInterface;
use shared::config::Config;
use shared::image_saver::storage::ImageStore;
use shared::queue::tasks::ImageTask;
use songbird::Songbird;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

pub type RedisConnection = Arc<RwLock<Option<redis::aio::MultiplexedConnection>>>;

pub struct BotData {
	pub config: Arc<Config>,
	pub bot_info: Arc<RwLock<Option<CurrentApplicationInfo>>>,
	pub anilist_cache: Arc<CacheInterface>,
	pub vndb_cache: Arc<CacheInterface>,
	pub steam_cache: Arc<CacheInterface>,
	pub already_launched: RwLock<bool>,
	pub apps: Arc<RwLock<HashMap<String, u32>>>,
	pub user_blacklist: Arc<RwLock<HashSet<String>>>,
	pub db_connection: Arc<DatabaseConnection>,
	pub manager: Arc<Songbird>,
	pub http_client: Arc<Client>,
	pub shard_manager: Arc<RwLock<HashMap<ShardId, Arc<parking_lot::RwLock<ShardRunnerInfo>>>>>,
	pub lavalink: Arc<RwLock<Option<Arc<LavalinkClient>>>>,
	pub shutdown_signal: Arc<tokio::sync::broadcast::Sender<()>>,
	pub vocal_session: Arc<RwLock<HashMap<(String, String), DateTime<Utc>>>>,
	pub user_color_update_count: Arc<AtomicUsize>,
	pub server_image_running: Arc<AtomicBool>,
	pub redis_connection: RedisConnection,
	pub user_color_task_tx: tokio::sync::mpsc::UnboundedSender<ImageTask>,
	pub server_image_task_tx: tokio::sync::mpsc::UnboundedSender<ImageTask>,
	pub image_store: Arc<dyn ImageStore>,
}
impl BotData {
	pub async fn get_hourly_usage(&self, command_name: String, user_id: String) -> u128 {
		let conn = self.db_connection.clone();
		let now = chrono::Utc::now();
		let hour_start = now
			.date_naive()
			.and_hms_opt(now.hour(), 0, 0)
			.unwrap()
			.and_utc();
		let hour_end = hour_start + chrono::Duration::hours(1);

		match shared::database::command_usage::Entity::find()
			.filter(shared::database::command_usage::Column::Command.eq(command_name))
			.filter(shared::database::command_usage::Column::User.eq(user_id))
			.filter(shared::database::command_usage::Column::UseTime.gte(hour_start))
			.filter(shared::database::command_usage::Column::UseTime.lt(hour_end))
			.count(&*conn)
			.await
		{
			Ok(count) => count as u128,
			Err(e) => {
				error!("DB error reading command usage: {}", e);
				0u128
			},
		}
	}

	pub async fn get_redis_connection(
		&self,
	) -> Option<tokio::sync::RwLockWriteGuard<'_, Option<redis::aio::MultiplexedConnection>>> {
		let mut guard = self.redis_connection.write().await;
		if guard.is_some() {
			return Some(guard);
		}

		let redis_url = self.config.queue.redis_url();
		match redis::Client::open(redis_url.as_str()) {
			Ok(client) => match client.get_multiplexed_async_connection().await {
				Ok(conn) => {
					info!(
						"Reconnected to Redis at {}:{}",
						self.config.queue.host, self.config.queue.port
					);
					*guard = Some(conn);
					Some(guard)
				},
				Err(e) => {
					warn!("Redis reconnect failed: {}", e);
					None
				},
			},
			Err(e) => {
				warn!("Redis client creation failed: {}", e);
				None
			},
		}
	}

	pub async fn increment_command_use_per_command(
		&self, command_name: String, user_id: String, _user_name: String,
	) {
		let conn = self.db_connection.clone();
		let now = chrono::Utc::now().naive_utc();

		let am = shared::database::command_usage::ActiveModel {
			command: Set(command_name.clone()),
			user: Set(user_id.clone()),
			use_time: Set(now),
		};

		if let Err(e) = am.insert(&*conn).await {
			error!("Failed to insert command usage in DB: {}", e);
		}
	}
}

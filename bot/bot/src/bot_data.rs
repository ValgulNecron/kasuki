use chrono::{DateTime, Timelike, Utc};
use dashmap::DashMap;
use futures::channel::mpsc::UnboundedSender;
use lavalink_rs::client::LavalinkClient;
use reqwest::Client;
use sea_orm::ActiveValue::Set;
use sea_orm::{
	ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
};
use serenity::all::{CurrentApplicationInfo, ShardId};
use serenity::gateway::{ShardRunnerInfo, ShardRunnerMessage};
use shared::cache::CacheInterface;
use shared::config::Config;
use songbird::Songbird;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::error;

pub struct BotData {
	pub config: Arc<Config>,
	pub bot_info: Arc<RwLock<Option<CurrentApplicationInfo>>>,
	pub anilist_cache: Arc<RwLock<CacheInterface>>,
	pub vndb_cache: Arc<RwLock<CacheInterface>>,
	pub already_launched: RwLock<bool>,
	pub apps: Arc<RwLock<HashMap<String, u128>>>,
	pub user_blacklist: Arc<RwLock<Vec<String>>>,
	pub db_connection: Arc<DatabaseConnection>,
	pub manager: Arc<Songbird>,
	pub http_client: Arc<Client>,
	pub shard_manager: Arc<
		RwLock<
			Option<Arc<DashMap<ShardId, (ShardRunnerInfo, UnboundedSender<ShardRunnerMessage>)>>>,
		>,
	>,
	pub lavalink: Arc<RwLock<Option<LavalinkClient>>>,
	pub shutdown_signal: Arc<tokio::sync::broadcast::Sender<()>>,
	pub vocal_session: Arc<RwLock<HashMap<(String, String), DateTime<Utc>>>>,
	pub user_color_update_count: Arc<AtomicUsize>,
	pub server_image_running: Arc<AtomicBool>,
}

impl BotData {
	pub async fn get_hourly_usage(&self, command_name: String, user_id: String) -> u128 {
		// Query database for command usage in the current hour
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

	// Insert command usage record to DB
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

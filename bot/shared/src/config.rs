use anyhow::{bail, Context, Result};
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use std::time::Duration;
use tracing::info;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
	pub bot: BotConfig,
	pub music: Option<MusicConfig>,
	pub db: DbConfig,
	pub image: ImageConfig,
	pub logging: LoggingConfig,
	pub ai: AICfg,
	pub task_intervals: TaskIntervalConfig,
	pub api: ApiConfig,
	pub cache: CacheConfig,
	pub queue: QueueConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CacheConfig {}

#[derive(Debug, Deserialize, Clone)]
pub struct QueueConfig {
	pub queue_type: String,
	pub host: String,
	pub port: u16,
	pub password: Option<String>,
}

impl QueueConfig {
	pub fn redis_url(&self) -> String {
		match self.password.as_deref() {
			Some(pw) if !pw.is_empty() => {
				// Percent-encode the password so special chars don't break URL parsing
				let encoded: String = pw
					.bytes()
					.map(|b| match b {
						b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
							String::from(b as char)
						},
						_ => format!("%{:02X}", b),
					})
					.collect();
				format!("redis://:{}@{}:{}", encoded, self.host, self.port)
			},
			_ => format!("redis://{}:{}", self.host, self.port),
		}
	}
}

#[derive(Debug, Deserialize, Clone)]
pub struct BotConfig {
	pub discord_token: String,
	pub bot_activity: String,
	pub remove_old_commands: bool,
	pub respect_premium: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MusicConfig {
	pub lavalink_hostname: String,
	pub lavalink_password: String,
	pub https: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DbConfig {
	pub db_type: String,
	pub host: Option<String>,
	pub port: Option<u16>,
	pub user: Option<String>,
	pub password: Option<String>,
	pub database: Option<String>,
	/// Maximum number of connections in the pool (default: 100)
	pub max_connections: Option<u32>,
	/// Minimum number of connections in the pool (default: 5)
	pub min_connections: Option<u32>,
	/// Connection timeout in seconds (default: 30)
	pub connect_timeout: Option<u64>,
	/// Idle timeout in seconds (default: 600)
	pub idle_timeout: Option<u64>,
}

impl DbConfig {
	/// Build a database connection URL from the config.
	pub fn get_url(&self) -> Result<String> {
		match self.db_type.as_str() {
			"postgresql" => {
				let host = self.host.as_deref().context("No database host provided")?;
				let port = self.port.context("No database port provided")?;
				let user = self.user.as_deref().context("No database user provided")?;
				let password = self
					.password
					.as_deref()
					.context("No database password provided")?;
				let db_name = self.database.as_deref().unwrap_or("kasuki");

				let param = serde_urlencoded::to_string(&[("user", user), ("password", password)])
					.context("Failed to encode database parameters")?;

				Ok(format!(
					"postgresql://{}:{}/{}?{}",
					host, port, db_name, param
				))
			},
			"sqlite" => {
				let path = self
					.database
					.as_deref()
					.context("No database path provided for SQLite")?;
				Ok(format!("sqlite://{}?mode=rwc", path))
			},
			other => bail!("Unsupported database type: {}", other),
		}
	}

	/// Create a SeaORM database connection with pool settings from config.
	pub async fn connect(&self) -> Result<DatabaseConnection> {
		let url = self.get_url()?;
		let mut opts = sea_orm::ConnectOptions::new(url);

		let max_connections = self.max_connections.unwrap_or(100);
		let min_connections = self.min_connections.unwrap_or(5);
		let connect_timeout = self.connect_timeout.unwrap_or(30);
		let idle_timeout = self.idle_timeout.unwrap_or(600);

		opts.max_connections(max_connections)
			.min_connections(min_connections)
			.connect_timeout(Duration::from_secs(connect_timeout))
			.idle_timeout(Duration::from_secs(idle_timeout))
			.sqlx_logging(false);

		info!(
			"Database pool config: max={}, min={}, connect_timeout={}s, idle_timeout={}s",
			max_connections, min_connections, connect_timeout, idle_timeout
		);

		sea_orm::Database::connect(opts)
			.await
			.context("Failed to connect to database")
	}
}

#[derive(Debug, Deserialize, Clone)]
pub struct ImageConfig {
	pub save_image: String,
	pub save_server: Option<String>,
	pub token: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
	pub log_level: String,
	pub max_log_retention: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AICfg {
	pub image: AICfgImage,
	pub question: AICfgQuestion,
	pub transcription: AICfgTranscription,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AICfgImage {
	pub ai_image_token: Option<String>,
	pub ai_image_base_url: Option<String>,
	pub ai_image_model: Option<String>,
	pub ai_image_quality: Option<String>,
	pub ai_image_size: Option<String>,
	pub ai_image_style: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AICfgQuestion {
	pub ai_question_token: Option<String>,
	pub ai_question_base_url: Option<String>,
	pub ai_question_model: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AICfgTranscription {
	pub ai_transcription_token: Option<String>,
	pub ai_transcription_base_url: Option<String>,
	pub ai_transcription_model: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TaskIntervalConfig {
	pub ping_update: u64,
	pub before_server_image: u64,
	pub server_image_update: u64,
	pub game_update: u64,
	pub bot_info: u64,
	pub blacklisted_user_update: u64,
	pub activity_check: u64,
	pub random_stats_update: u64,
	pub anisong_update: u64,
	pub bot_info_update: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApiConfig {
	pub enabled: bool,
	pub port: u16,
	#[serde(default)]
	pub debug: bool,
	pub allowed_domain: Option<String>,
	pub oauth: OAuthConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OAuthConfig {
	pub discord_client_id: String,
	pub discord_client_secret: String,
	pub discord_redirect_uri: String,
	pub frontend_url: String,
	pub jwt_secret: String, // New field for JWT secret
}

impl Config {
	pub fn new() -> Result<Self> {
		let config = std::fs::read_to_string("config.toml")?;
		let config: Config = toml::from_str(&config)?;
		Ok(config)
	}

	pub fn load_from_path(path: &str) -> Result<Self> {
		let config = std::fs::read_to_string(path)?;
		let config: Config = toml::from_str(&config)?;
		Ok(config)
	}
}

/// Lightweight config for the worker binary — only the sections it needs.
#[derive(Debug, Deserialize, Clone)]
pub struct WorkerConfig {
	pub bot: BotConfig,
	pub db: DbConfig,
	pub task_intervals: TaskIntervalConfig,
}

impl WorkerConfig {
	pub fn new() -> Result<Self> {
		let config = std::fs::read_to_string("config.toml")?;
		let config: WorkerConfig = toml::from_str(&config)?;
		Ok(config)
	}
}

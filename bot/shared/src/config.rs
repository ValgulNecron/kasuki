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
	pub sentry_url: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CacheConfig {
	/// "memory" (default) or "redis"
	#[serde(default = "default_cache_type")]
	pub cache_type: String,
	/// Redis host (only used when cache_type = "redis")
	pub host: Option<String>,
	/// Redis port (only used when cache_type = "redis")
	pub port: Option<u16>,
	/// Redis password (only used when cache_type = "redis")
	pub password: Option<String>,
	/// TTL in seconds for cache entries (default: 3600)
	#[serde(default = "default_cache_ttl")]
	pub ttl_secs: u64,
	/// Maximum number of entries for the in-memory backend (default: 2 000)
	#[serde(default = "default_cache_max_capacity")]
	pub max_capacity: u64,
}

fn default_cache_type() -> String {
	"memory".to_string()
}

fn default_cache_ttl() -> u64 {
	3600
}

fn default_cache_max_capacity() -> u64 {
	2_000
}

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
	/// Maximum number of tasks processed concurrently. Defaults to 1.
	#[serde(default = "default_max_workers")]
	pub max_workers: usize,
	#[serde(default)]
	pub storage: StorageConfig,
}

fn default_max_workers() -> usize {
	let cpus = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
	(cpus / 10).max(1)
}

#[derive(Debug, Deserialize, Clone)]
pub struct StorageConfig {
	/// "local" (default) or "s3"
	#[serde(default = "default_storage_type")]
	pub storage_type: String,
	/// Base path for local storage (default: "./images")
	pub local_path: Option<String>,
	/// S3-compatible endpoint URL
	pub s3_endpoint: Option<String>,
	/// S3 bucket name
	pub s3_bucket: Option<String>,
	/// S3 region (default: "us-east-1")
	pub s3_region: Option<String>,
	/// S3 access key
	pub s3_access_key: Option<String>,
	/// S3 secret key
	pub s3_secret_key: Option<String>,
}

fn default_storage_type() -> String {
	"local".to_string()
}

impl Default for StorageConfig {
	fn default() -> Self {
		Self {
			storage_type: default_storage_type(),
			local_path: None,
			s3_endpoint: None,
			s3_bucket: None,
			s3_region: None,
			s3_access_key: None,
			s3_secret_key: None,
		}
	}
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
	#[serde(default)]
	pub rate_limits: AiRateLimits,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AiRateLimits {
	#[serde(default = "default_free_limit")]
	pub free_images: usize,
	#[serde(default = "default_free_limit")]
	pub free_questions: usize,
	#[serde(default = "default_free_limit")]
	pub free_translations: usize,
	#[serde(default = "default_free_limit")]
	pub free_transcripts: usize,
	#[serde(default = "default_paid_multiplier")]
	pub paid_image_multiplier: f64,
	#[serde(default = "default_paid_multiplier")]
	pub paid_question_multiplier: f64,
	#[serde(default = "default_paid_multiplier")]
	pub paid_translation_multiplier: f64,
	#[serde(default = "default_paid_multiplier")]
	pub paid_transcript_multiplier: f64,
}

fn default_free_limit() -> usize {
	5
}

fn default_paid_multiplier() -> f64 {
	5.0
}

impl Default for AiRateLimits {
	fn default() -> Self {
		Self {
			free_images: default_free_limit(),
			free_questions: default_free_limit(),
			free_translations: default_free_limit(),
			free_transcripts: default_free_limit(),
			paid_image_multiplier: default_paid_multiplier(),
			paid_question_multiplier: default_paid_multiplier(),
			paid_translation_multiplier: default_paid_multiplier(),
			paid_transcript_multiplier: default_paid_multiplier(),
		}
	}
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
	#[serde(default = "default_rate_limit")]
	pub rate_limit_per_minute: u32,
	#[serde(default)]
	pub cache: ApiCacheConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApiCacheConfig {
	#[serde(default = "default_user_cache_capacity")]
	pub user_cache_capacity: u64,
	#[serde(default = "default_user_cache_ttl")]
	pub user_cache_ttl_secs: u64,
	#[serde(default = "default_auth_code_capacity")]
	pub auth_code_capacity: u64,
	#[serde(default = "default_auth_code_ttl")]
	pub auth_code_ttl_secs: u64,
	#[serde(default = "default_oauth_state_capacity")]
	pub oauth_state_capacity: u64,
	#[serde(default = "default_oauth_state_ttl")]
	pub oauth_state_ttl_secs: u64,
}

fn default_rate_limit() -> u32 {
	10
}
fn default_user_cache_capacity() -> u64 {
	10_000
}
fn default_user_cache_ttl() -> u64 {
	86400
}
fn default_auth_code_capacity() -> u64 {
	1_000
}
fn default_auth_code_ttl() -> u64 {
	300
}
fn default_oauth_state_capacity() -> u64 {
	1_000
}
fn default_oauth_state_ttl() -> u64 {
	600
}

impl Default for ApiCacheConfig {
	fn default() -> Self {
		Self {
			user_cache_capacity: default_user_cache_capacity(),
			user_cache_ttl_secs: default_user_cache_ttl(),
			auth_code_capacity: default_auth_code_capacity(),
			auth_code_ttl_secs: default_auth_code_ttl(),
			oauth_state_capacity: default_oauth_state_capacity(),
			oauth_state_ttl_secs: default_oauth_state_ttl(),
		}
	}
}

#[derive(Debug, Deserialize, Clone)]
pub struct OAuthConfig {
	pub discord_client_id: String,
	pub discord_client_secret: String,
	pub discord_redirect_uri: String,
	pub frontend_url: String,
	pub jwt_secret: String,
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
	pub cache: CacheConfig,
	pub sentry_url: Option<String>,
}

impl WorkerConfig {
	pub fn new() -> Result<Self> {
		let config = std::fs::read_to_string("config.toml")?;
		let config: WorkerConfig = toml::from_str(&config)?;
		Ok(config)
	}
}

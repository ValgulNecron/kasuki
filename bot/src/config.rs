use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
	pub bot: BotConfig,
	pub music: Option<MusicConfig>,
	pub db: DbConfig,
	pub image: ImageConfig,
	pub logging: LoggingConfig,
	pub ai: AICfg,
	pub task_intervals: TaskIntervalConfig,
	pub cache: CacheConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CacheConfig {
	pub cache_type: String,
	pub host: Option<String>,
	pub port: Option<u16>,
	pub user: Option<String>,
	pub password: Option<String>,
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
	pub ai_token: String,
	pub ai_base_url: String,
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
	pub cache_update: u64,
	pub bot_info: u64,
	pub blacklisted_user_update: u64,
	pub activity_check: u64,
	pub random_stats_update: u64,
	pub anisong_update: u64,
	pub bot_info_update: u64,
}

impl Config {
	pub fn new() -> Result<Self> {
		let config = std::fs::read_to_string("config.toml")?;
		let config: Config = toml::from_str(&config)?;
		Ok(config)
	}
}

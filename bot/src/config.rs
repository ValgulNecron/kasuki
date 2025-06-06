use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]

pub struct Config {
	pub bot: BotConfig,
	pub music: MusicConfig,
	pub db: DbConfig,
	pub image: ImageConfig,
	pub logging: LoggingConfig,
	pub ai: AICfg,
	pub task_intervals: TaskIntervalConfig,
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
	pub bot_info_update: u64
}

impl Default for Config {
	fn default() -> Self {
		Config {
			bot: BotConfig {
				discord_token: "".to_string(),
				bot_activity: "".to_string(),
				remove_old_commands: false,
				respect_premium: false,
			},
			music: MusicConfig {
				lavalink_hostname: "".to_string(),
				lavalink_password: "".to_string(),
				https: false,
			},
			db: DbConfig {
				db_type: "sqlite".to_string(),
				host: None,
				port: None,
				user: None,
				password: None,
				database: Some("kasuki".to_string()),
			},
			image: ImageConfig {
				save_image: "local".to_string(),
				save_server: None,
				token: None,
			},
			logging: LoggingConfig {
				log_level: "info".to_string(),
				max_log_retention: 30,
			},
			ai: AICfg {
				ai_token: "".to_string(),
				ai_base_url: "".to_string(),
				image: AICfgImage {
					ai_image_token: None,
					ai_image_base_url: None,
					ai_image_model: None,
					ai_image_quality: None,
					ai_image_size: None,
					ai_image_style: None,
				},
				question: AICfgQuestion {
					ai_question_token: None,
					ai_question_base_url: None,
					ai_question_model: None,
				},
				transcription: AICfgTranscription {
					ai_transcription_token: None,
					ai_transcription_base_url: None,
					ai_transcription_model: None,
				},
			},
			task_intervals: TaskIntervalConfig {
				ping_update: 600,
				before_server_image: 1_800,
				server_image_update: 21_600,
				game_update: 86_400,
				cache_update: 259_200,
				bot_info: 1_800,
				blacklisted_user_update: 3600,
				activity_check: 1,
				random_stats_update: 86_400,
				anisong_update: 604_800,
				bot_info_update: 1_800,
			},
		}
	}
}

impl Config {
	pub fn set_default_value_on_none(&mut self) {
		if self.ai.image.ai_image_token.is_none()
			|| self
				.ai
				.image
				.ai_image_token
				.clone()
				.unwrap_or_default()
				.is_empty()
		{
			self.ai.image.ai_image_token = Some(self.ai.ai_token.clone());
		}

		if self.ai.image.ai_image_base_url.is_none()
			|| self
				.ai
				.image
				.ai_image_base_url
				.clone()
				.unwrap_or_default()
				.is_empty()
		{
			self.ai.image.ai_image_base_url = Some(self.ai.ai_base_url.clone());
		}

		if self.ai.image.ai_image_model.is_none()
			|| self
				.ai
				.image
				.ai_image_model
				.clone()
				.unwrap_or_default()
				.is_empty()
		{
			self.ai.image.ai_image_model = Some("dall-e-3".to_string());
		}

		if self.ai.question.ai_question_token.is_none()
			|| self
				.ai
				.question
				.ai_question_token
				.clone()
				.unwrap_or_default()
				.is_empty()
		{
			self.ai.question.ai_question_token = Some(self.ai.ai_token.clone());
		}

		if self.ai.question.ai_question_base_url.is_none()
			|| self
				.ai
				.question
				.ai_question_base_url
				.clone()
				.unwrap_or_default()
				.is_empty()
		{
			self.ai.question.ai_question_base_url = Some(self.ai.ai_base_url.clone());
		}

		if self.ai.question.ai_question_model.is_none()
			|| self
				.ai
				.question
				.ai_question_model
				.clone()
				.unwrap_or_default()
				.is_empty()
		{
			self.ai.question.ai_question_model = Some("gpt-3.5-turbo".to_string());
		}

		if self.ai.transcription.ai_transcription_token.is_none()
			|| self
				.ai
				.transcription
				.ai_transcription_token
				.clone()
				.unwrap_or_default()
				.is_empty()
		{
			self.ai.transcription.ai_transcription_token = Some(self.ai.ai_token.clone());
		}

		if self.ai.transcription.ai_transcription_base_url.is_none()
			|| self
				.ai
				.transcription
				.ai_transcription_base_url
				.clone()
				.unwrap_or_default()
				.is_empty()
		{
			self.ai.transcription.ai_transcription_base_url = Some(self.ai.ai_base_url.clone());
		}

		if self.ai.transcription.ai_transcription_model.is_none()
			|| self
				.ai
				.transcription
				.ai_transcription_model
				.clone()
				.unwrap_or_default()
				.is_empty()
		{
			self.ai.transcription.ai_transcription_model = Some("whisper-1".to_string());
		}
	}
}

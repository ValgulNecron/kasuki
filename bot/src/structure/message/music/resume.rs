use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ResumeLocalised {
	pub title: String,
	pub error_no_voice: String,
	pub success: String,
}

use anyhow::Result;

pub async fn load_localization_resume(
	guild_id: String, db_config: DbConfig,
) -> Result<ResumeLocalised> {
	let path = "json/message/music/resume.json";

	load_localization(guild_id, path, db_config).await
}

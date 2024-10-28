use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct CommandUsageLocalised {
	pub title: String,
	pub no_usage: String,
	pub command_usage: String,
}

use anyhow::Result;

pub async fn load_localization_command_usage(
	guild_id: String, db_config: DbConfig,
) -> Result<CommandUsageLocalised> {
	let path = "json/message/user/command_usage.json";

	load_localization(guild_id, path, db_config).await
}

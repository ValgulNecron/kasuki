use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]

pub struct GuildLocalised {
	pub guild_id: String,
	pub guild_name: String,
	pub member: String,
	pub online: String,
	pub lang: String,
	pub premium: String,
	pub sub: String,
	pub nsfw: String,
	pub creation_date: String,
	pub owner: String,
	pub roles: String,
	pub channels: String,
	pub verification_level: String,
}

use anyhow::Result;

pub async fn load_localization_guild(
	guild_id: String, db_config: DbConfig,
) -> Result<GuildLocalised> {
	let path = "json/message/server/guild.json";

	load_localization(guild_id, path, db_config).await
}

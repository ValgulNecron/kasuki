use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]

pub struct NewMember {
	pub welcome: String,
}

use anyhow::Result;

pub async fn load_localization_new_member(
	guild_id: String, db_config: DbConfig,
) -> Result<NewMember> {
	let path = "json/message/new_member.json";

	load_localization(guild_id, path, db_config).await
}

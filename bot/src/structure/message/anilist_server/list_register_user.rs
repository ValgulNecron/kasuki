use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct ListUserLocalised {
	pub title: String,
	pub next: String,
	pub previous: String,
}

use anyhow::Result;

pub async fn load_localization_list_user(
	guild_id: String, db_config: DbConfig,
) -> Result<ListUserLocalised> {
	let path = "json/message/anilist_server/list_register_user.json";

	load_localization(guild_id, path, db_config).await
}

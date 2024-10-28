use crate::config::DbConfig;
use serde::{Deserialize, Serialize};

// Importing necessary libraries and modules
use crate::structure::message::common::load_localization;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StaffLocalised {
	pub media: String,

	pub va: String,

	pub date_of_birth: String,

	pub date_of_death: String,

	pub hometown: String,

	pub lang: String,

	pub occupation: String,

	pub age: String,

	pub gender: String,
}

use anyhow::Result;

pub async fn load_localization_staff(
	guild_id: String, db_config: DbConfig,
) -> Result<StaffLocalised> {
	let path = "json/message/anilist_user/staff.json";

	load_localization(guild_id, path, db_config).await
}

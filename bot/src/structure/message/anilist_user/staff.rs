use crate::config::DbConfig;
use serde::{Deserialize, Serialize};

// Importing necessary libraries and modules
use crate::structure::message::common::load_localization;

#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct StaffLocalised {
	pub field1_title: String,
	pub field2_title: String,
	pub desc: String,
	pub date_of_birth: String,
	pub date_of_death: String,
}

use anyhow::Result;

pub async fn load_localization_staff(
	guild_id: String, db_config: DbConfig,
) -> Result<StaffLocalised> {
	let path = "json/message/anilist_user/staff.json";

	load_localization(guild_id, path, db_config).await
}

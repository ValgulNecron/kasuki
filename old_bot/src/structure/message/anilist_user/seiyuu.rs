use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Importing necessary libraries and modules
use crate::structure::message::common::load_localization;

#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct SeiyuuLocalised {
	pub title: String,
}

use anyhow::Result;
use sea_orm::DatabaseConnection;

pub async fn load_localization_seiyuu(
	guild_id: String, db_connection: Arc<DatabaseConnection>,
) -> Result<SeiyuuLocalised> {
	let path = "json/message/anilist_user/seiyuu.json";

	load_localization(guild_id, path, db_connection).await
}

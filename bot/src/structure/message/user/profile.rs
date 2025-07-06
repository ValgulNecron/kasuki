use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct ProfileLocalised {
	pub title: String,
	pub id: String,
	pub creation_date: String,
	pub joined_date: String,
	pub bot: String,
	pub nitro: String,
	pub system: String,
	pub public_flag: String,
	pub premium: String,
}

use anyhow::Result;
use sea_orm::DatabaseConnection;

pub async fn load_localization_profile(
	guild_id: String, db_connection: Arc<DatabaseConnection>,
) -> Result<ProfileLocalised> {
	let path = "json/message/user/profile.json";

	load_localization(guild_id, path, db_connection).await
}

use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, Clone)]

pub struct CharacterLocalised {
	pub date_of_birth: String,

	pub age: String,

	pub gender: String,

	pub fav: String,
	pub blood_type: String,
}

use anyhow::Result;
use sea_orm::DatabaseConnection;

pub async fn load_localization_character(
	guild_id: String, db_connection: Arc<DatabaseConnection>,
) -> Result<CharacterLocalised> {
	let path = "json/message/anilist_user/character.json";

	load_localization(guild_id, path, db_connection).await
}

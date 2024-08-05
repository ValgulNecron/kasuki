#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JoinLocalised {
	pub title: String,

	pub already_in: String,
}

use std::error::Error;
use serde::{Serialize, Deserialize};
use crate::structure::message::common::load_localization;

pub async fn load_localization_join_localised(
	guild_id: String,
	db_type: String,
) -> Result<JoinLocalised, Box<dyn Error>> {
	let path = "json/message/audio/join.json";
	load_localization(guild_id, path, db_type).await
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayLocalised {
	pub error: String,

	pub no_option: String,

	pub now_playing: String,
}

use std::error::Error;
use serde::{Serialize, Deserialize};
use crate::structure::message::common::load_localization;

pub async fn load_localization_play_localised(
	guild_id: String,
	db_type: String,
) -> Result<PlayLocalised, Box<dyn Error>> {
	let path = "json/message/audio/play.json";
	load_localization(guild_id, path, db_type).await
}
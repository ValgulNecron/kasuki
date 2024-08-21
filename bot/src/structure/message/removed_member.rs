use std::error::Error;

use crate::config::BotConfigDetails;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RemovedMember {
	pub bye: String,

	pub ban: String,

	pub kick: String,

	pub ban_for: String,

	pub kick_for: String,
}
pub async fn load_localization_removed_member(
    guild_id: String,
    db_config: BotConfigDetails,
) -> Result<RemovedMember, Box<dyn Error>> {
    let path = "json/message/removed_member.json";
    load_localization(guild_id, path, db_config).await
}

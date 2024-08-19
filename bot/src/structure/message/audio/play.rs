#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayLocalised {
    pub error: String,

    pub now_playing: String,
}

use crate::config::BotConfigDetails;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};
use std::error::Error;

pub async fn load_localization_play_localised(
    guild_id: String,
    db_config: BotConfigDetails,
) -> Result<PlayLocalised, Box<dyn Error>> {
    let path = "json/message/audio/play.json";
    load_localization(guild_id, path, db_config).await
}
use std::error::Error;

use crate::config::BotConfigDetails;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameLocalised {
    pub released: String,

    pub platforms: String,

    pub playtime: String,

    pub tags: String,

    pub developers: String,

    pub staff: String,

    pub characters: String,
}

pub async fn load_localization_game(
    guild_id: String,
    db_config: BotConfigDetails,
) -> Result<GameLocalised, Box<dyn Error>> {
    let path = "json/message/vn/game.json";
    load_localization(guild_id, path, db_config).await
}

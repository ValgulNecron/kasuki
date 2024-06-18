use serde::{Deserialize, Serialize};

use crate::helper::error_management::error_enum::AppError;
use crate::structure::message::common::load_localization;

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

pub async fn load_localization_game(guild_id: String) -> Result<GameLocalised, AppError> {
    let path = "json/message/vn/game.json";
    load_localization(guild_id, path).await
}

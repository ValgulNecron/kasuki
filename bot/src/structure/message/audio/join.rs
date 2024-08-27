#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JoinLocalised {
    pub title: String,

    pub already_in: String,
}

use crate::config::BotConfigDetails;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};
use std::error::Error;

pub async fn load_localization_join_localised(
    guild_id: String,
    db_config: BotConfigDetails,
) -> Result<JoinLocalised, Box<dyn Error>> {
    let path = "json/message/audio/join.json";
    load_localization(guild_id, path, db_config).await
}

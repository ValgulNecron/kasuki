use crate::config::BotConfigDetails;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GivePremiumLocalised {
    pub success: String,
}

pub async fn load_localization_give_premium_sub(
    guild_id: String,
    db_config: BotConfigDetails,
) -> Result<GivePremiumLocalised, Box<dyn Error>> {
    let path = "json/message/management/give_premium_sub.json";
    load_localization(guild_id, path, db_config).await
}

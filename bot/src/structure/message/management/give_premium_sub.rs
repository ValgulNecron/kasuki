use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct GivePremiumLocalised {
    pub success: String,
}

use anyhow::Result;

pub async fn load_localization_give_premium_sub(
    guild_id: String,
    db_config: DbConfig,
) -> Result<GivePremiumLocalised> {
    let path = "json/message/management/give_premium_sub.json";

    load_localization(guild_id, path, db_config).await
}

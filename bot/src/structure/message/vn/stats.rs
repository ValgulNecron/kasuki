use std::error::Error;

use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]

pub struct StatsLocalised {
    pub title: String,

    pub chars: String,

    pub producer: String,

    pub release: String,

    pub staff: String,

    pub tags: String,

    pub traits: String,

    pub vns: String,

    pub api: String,
}

pub async fn load_localization_stats(
    guild_id: String,
    db_config: DbConfig,
) -> Result<StatsLocalised, Box<dyn Error>> {

    let path = "json/message/vn/stats.json";

    load_localization(guild_id, path, db_config).await
}

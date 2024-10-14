use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct PingLocalised {
    pub title: String,
    pub desc: String,
}

use anyhow::Result;

pub async fn load_localization_ping(
    guild_id: String,
    db_config: DbConfig,
) -> Result<PingLocalised> {
    let path = "json/message/bot/ping.json";

    load_localization(guild_id, path, db_config).await
}

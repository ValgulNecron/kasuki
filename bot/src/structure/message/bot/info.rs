use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct InfoLocalised {
    pub title: String,
    pub desc: String,
    pub bot_name: String,
    pub bot_id: String,
    pub server_count: String,
    pub user_count: String,
    pub creation_date: String,
    pub shard: String,
    pub shard_count: String,
    pub version: String,
    pub library: String,
    pub footer: String,
    pub button_see_on_github: String,
    pub button_official_website: String,
    pub button_official_discord: String,
    pub button_add_the_bot: String,
    pub button_add_the_beta_bot: String,
    pub app_installation_count: String,
}

use anyhow::Result;

pub async fn load_localization_info(
    guild_id: String,
    db_config: DbConfig,
) -> Result<InfoLocalised> {
    let path = "json/message/bot/info.json";

    load_localization(guild_id, path, db_config).await
}

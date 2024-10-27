use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct ListActivityLocalised {
    pub title: String,
    pub next: String,
    pub previous: String,
}

use anyhow::Result;

pub async fn load_localization_list_activity(
    guild_id: String,
    db_config: DbConfig,
) -> Result<ListActivityLocalised> {
    let path = "json/message/anilist_server/list_all_activity.json";

    load_localization(guild_id, path, db_config).await
}

use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct AvatarLocalised {
    pub title: String,
    pub server_title: String,
}

use anyhow::Result;

pub async fn load_localization_avatar(
    guild_id: String,
    db_config: DbConfig,
) -> Result<AvatarLocalised> {
    let path = "json/message/user/avatar.json";

    load_localization(guild_id, path, db_config).await
}

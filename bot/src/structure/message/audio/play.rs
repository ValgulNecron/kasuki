#[derive(Serialize, Deserialize, Debug, Clone)]

pub struct PlayLocalised {
    pub error: String,

    pub now_playing: String,
}

use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use anyhow::Result;
use serde::{Deserialize, Serialize};

pub async fn load_localization_play_localised(
    guild_id: String,
    db_config: DbConfig,
) -> Result<PlayLocalised> {
    let path = "json/message/audio/play.json";

    load_localization(guild_id, path, db_config).await
}

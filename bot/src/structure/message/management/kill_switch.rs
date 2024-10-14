use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct KillSwitchLocalised {
    pub on: String,
    pub off: String,
}

use anyhow::Result;

pub async fn load_localization_kill_switch(
    guild_id: String,
    db_config: DbConfig,
) -> Result<KillSwitchLocalised> {
    let path = "json/message/management/kill_switch.json";

    load_localization(guild_id, path, db_config).await
}

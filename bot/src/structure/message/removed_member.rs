use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]

pub struct RemovedMember {
    pub bye: String,

    pub ban: String,

    pub kick: String,

    pub ban_for: String,

    pub kick_for: String,
}

use anyhow::Result;

pub async fn load_localization_removed_member(
    guild_id: String,
    db_config: DbConfig,
) -> Result<RemovedMember> {

    let path = "json/message/removed_member.json";

    load_localization(guild_id, path, db_config).await
}

use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct LevelsStatsLocalised {
    pub vocal: String,
    pub vocal_len: String,
    pub message: String,
    pub message_len: String,
}

use anyhow::Result;
use sea_orm::DatabaseConnection;

pub async fn load_localization_levels_stats(
    guild_id: String, db_connection: Arc<DatabaseConnection>,
) -> Result<LevelsStatsLocalised> {
    let path = "json/message/levels/stats.json";

    load_localization(guild_id, path, db_connection).await
}

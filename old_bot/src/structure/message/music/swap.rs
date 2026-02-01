use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SwapLocalised {
    pub title: String,
    pub error_no_voice: String,
    pub error_same_index: String,
    pub error_max_index: String,
    pub success: String,
}

use anyhow::Result;
use sea_orm::DatabaseConnection;

pub async fn load_localization_swap(
    guild_id: String, db_connection: Arc<DatabaseConnection>,
) -> Result<SwapLocalised> {
    let path = "json/message/music/swap.json";

    load_localization(guild_id, path, db_connection).await
}

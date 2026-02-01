use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct DeleteActivityLocalised {
    pub success: String,
    pub success_desc: String,
}

use anyhow::Result;
use sea_orm::DatabaseConnection;

pub async fn load_localization_delete_activity(
    guild_id: String, db_connection: Arc<DatabaseConnection>,
) -> Result<DeleteActivityLocalised> {
    let path = "json/message/admin/anilist/delete_activity.json";

    load_localization(guild_id, path, db_connection).await
}

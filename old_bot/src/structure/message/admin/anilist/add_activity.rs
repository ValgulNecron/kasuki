use crate::structure::message::common::load_localization;
use anyhow::Result;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct AddActivityLocalised {
    pub success: String,
    pub fail: String,
    pub fail_desc: String,
    pub success_desc: String,
}

pub async fn load_localization_add_activity(
    guild_id: String, db_connection: Arc<DatabaseConnection>,
) -> Result<AddActivityLocalised> {
    let path = "json/message/admin/anilist/add_activity.json";

    load_localization(guild_id, path, db_connection).await
}

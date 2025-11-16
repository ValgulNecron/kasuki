use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct ListActivityLocalised {
    pub title: String,
    pub next: String,
    pub previous: String,
}

use anyhow::Result;
use sea_orm::DatabaseConnection;

pub async fn load_localization_list_activity(
    guild_id: String, db_connection: Arc<DatabaseConnection>,
) -> Result<ListActivityLocalised> {
    let path = "json/message/anilist_server/list_all_activity.json";

    load_localization(guild_id, path, db_connection).await
}

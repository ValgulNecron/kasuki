use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct ImageLocalised {
    pub title: String,
}

use anyhow::Result;
use sea_orm::DatabaseConnection;

pub async fn load_localization_image(
    guild_id: String, db_connection: Arc<DatabaseConnection>,
) -> Result<ImageLocalised> {
    let path = "json/message/ai/image.json";

    load_localization(guild_id, path, db_connection).await
}

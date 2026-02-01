use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct RandomImageLocalised {
    pub title: String,
}

use anyhow::Result;
use sea_orm::DatabaseConnection;

pub async fn load_localization_random_image(
    guild_id: String, db_connection: Arc<DatabaseConnection>,
) -> Result<RandomImageLocalised> {
    let path = "json/message/anime/random_image.json";

    load_localization(guild_id, path, db_connection).await
}

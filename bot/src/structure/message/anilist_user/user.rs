use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Importing necessary libraries and modules
use crate::structure::message::common::load_localization;

#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct UserLocalised {
    pub manga: String,
    pub anime: String,
    pub week: String,
    pub day: String,
    pub hour: String,
    pub minute: String,
    pub weeks: String,
    pub days: String,
    pub hours: String,
    pub minutes: String,
}

use anyhow::Result;
use sea_orm::DatabaseConnection;

pub async fn load_localization_user(
    guild_id: String, db_connection: Arc<DatabaseConnection>,
) -> Result<UserLocalised> {
    let path = "json/message/anilist_user/user.json";

    load_localization(guild_id, path, db_connection).await
}

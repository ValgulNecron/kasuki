use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, Clone)]

pub struct UserLocalised {
    pub title: String,

    pub id: String,

    pub name: String,

    pub playtimesum: String,

    pub playtime: String,
}

use anyhow::Result;
use sea_orm::DatabaseConnection;

pub async fn load_localization_user(
    guild_id: String, db_connection: Arc<DatabaseConnection>,
) -> Result<UserLocalised> {
    let path = "json/message/vn/user.json";

    load_localization(guild_id, path, db_connection).await
}

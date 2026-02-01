use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Importing necessary libraries and modules
use crate::structure::message::common::load_localization;

#[derive(Serialize, Deserialize, Debug, Clone)]

pub struct MediaLocalised {
    pub genre: String,

    pub tag: String,

    pub staffs: String,

    pub characters: String,

    pub format: String,

    pub source: String,

    pub start_date: String,

    pub end_date: String,

    pub fav: String,

    pub duration: String,
    pub chapter: String,

    pub minutes: String,

    pub song: String,
}

use anyhow::Result;
use sea_orm::DatabaseConnection;

pub async fn load_localization_media(
    guild_id: String, db_connection: Arc<DatabaseConnection>,
) -> Result<MediaLocalised> {
    let path = "json/message/anilist_user/media.json";

    load_localization(guild_id, path, db_connection).await
}

use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, Clone)]

pub struct StaffLocalised {
    pub main: String,

    pub aid: String,

    pub gender: String,

    pub lang: String,
}

use anyhow::Result;
use sea_orm::DatabaseConnection;

pub async fn load_localization_staff(
    guild_id: String, db_connection: Arc<DatabaseConnection>,
) -> Result<StaffLocalised> {
    let path = "json/message/vn/staff.json";

    load_localization(guild_id, path, db_connection).await
}

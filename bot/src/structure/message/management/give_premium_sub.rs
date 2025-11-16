use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct GivePremiumLocalised {
    pub success: String,
}

use anyhow::Result;
use sea_orm::DatabaseConnection;

pub async fn load_localization_give_premium_sub(
    guild_id: String, db_connection: Arc<DatabaseConnection>,
) -> Result<GivePremiumLocalised> {
    let path = "json/message/management/give_premium_sub.json";

    load_localization(guild_id, path, db_connection).await
}

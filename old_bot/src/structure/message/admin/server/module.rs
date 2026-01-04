use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct ModuleLocalised {
    pub on: String,
    pub off: String,
}

use anyhow::Result;
use sea_orm::DatabaseConnection;

pub async fn load_localization_module_activation(
    guild_id: String, db_connection: Arc<DatabaseConnection>,
) -> Result<ModuleLocalised> {
    let path = "json/message/admin/server/module.json";

    load_localization(guild_id, path, db_connection).await
}

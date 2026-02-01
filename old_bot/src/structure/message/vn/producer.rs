use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, Clone)]

pub struct ProducerLocalised {
    pub lang: String,

    pub prod_type: String,
    pub aliases: String,
}

use anyhow::Result;
use sea_orm::DatabaseConnection;

pub async fn load_localization_producer(
    guild_id: String, db_connection: Arc<DatabaseConnection>,
) -> Result<ProducerLocalised> {
    let path = "json/message/vn/producer.json";

    load_localization(guild_id, path, db_connection).await
}

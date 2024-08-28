use std::error::Error;

use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProducerLocalised {
    pub lang: String,

    pub prod_type: String,
    pub aliases: String,
}

pub async fn load_localization_producer(
    guild_id: String,
    db_config: DbConfig,
) -> Result<ProducerLocalised, Box<dyn Error>> {
    let path = "json/message/vn/producer.json";
    load_localization(guild_id, path, db_config).await
}

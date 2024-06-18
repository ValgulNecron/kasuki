use crate::helper::error_management::error_enum::AppError;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProducerLocalised {
    pub lang: String,

    pub prod_type: String,
    pub aliases: String,
}

pub async fn load_localization_producer(guild_id: String) -> Result<ProducerLocalised, AppError> {
    let path = "json/message/vn/producer.json";
    load_localization(guild_id, path).await
}

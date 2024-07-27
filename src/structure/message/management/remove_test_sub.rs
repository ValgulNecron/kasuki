use std::error::Error;
use serde::{Deserialize, Serialize};
use crate::structure::message::common::load_localization;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RemoveTestLocalised {
    pub success: String,
}

pub async fn load_localization_remove_test_sub(
    guild_id: String,
    db_type: String,
) -> Result<RemoveTestLocalised, Box<dyn Error>> {
    let path = "json/message/management/remove_test_sub.json";
    load_localization(guild_id, path, db_type).await
}
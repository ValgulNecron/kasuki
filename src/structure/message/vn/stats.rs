#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StatsLocalised {
    pub title: String,

    pub chars: String,

    pub producer: String,

    pub release: String,

    pub staff: String,

    pub tags: String,

    pub traits: String,

    pub vns: String,

    pub api: String,
}
use crate::helper::error_management::error_enum::AppError;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

pub async fn load_localization_stats(guild_id: String) -> Result<StatsLocalised, AppError> {
    let path = "json/message/vn/stats.json";
    load_localization(guild_id, path).await
}

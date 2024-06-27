use crate::helper::error_management::error_enum::AppError;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserLocalised {
    pub title: String,

    pub id: String,

    pub name: String,

    pub playtimesum: String,

    pub playtime: String,
}

pub async fn load_localization_user(
    guild_id: String,
    db_type: String,
) -> Result<UserLocalised, AppError> {
    let path = "json/message/vn/user.json";
    load_localization(guild_id, path, db_type).await
}

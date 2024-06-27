#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StaffLocalised {
    pub main: String,

    pub aid: String,

    pub gender: String,

    pub lang: String,
}
use crate::helper::error_management::error_enum::AppError;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

pub async fn load_localization_staff(
    guild_id: String,
    db_type: String,
) -> Result<StaffLocalised, AppError> {
    let path = "json/message/vn/staff.json";
    load_localization(guild_id, path, db_type).await
}

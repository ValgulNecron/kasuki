#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StaffLocalised {
    pub main: String,

    pub aid: String,

    pub gender: String,

    pub lang: String,
}
use crate::helper::error_management::error_enum::AppError;
use crate::structure::message::common::load_localization;
use crate::structure::message::vn::producer::ProducerLocalised;
use serde::{Deserialize, Serialize};

pub async fn load_localization_staff(guild_id: String) -> Result<StaffLocalised, AppError> {
    let path = "json/message/vn/staff.json";
    load_localization(guild_id, path).await
}

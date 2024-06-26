use serde::{Deserialize, Serialize};

use crate::helper::error_management::error_enum::AppError;
use crate::structure::message::common::load_localization;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CharacterLocalised {
    pub blood_type: String,

    pub height: String,

    pub weight: String,

    pub age: String,

    pub bust: String,

    pub waist: String,

    pub hip: String,

    pub cup: String,

    pub sex: String,

    pub birthday: String,

    pub vns: String,

    pub traits: String,
}

pub async fn load_localization_character(
    guild_id: String,
    db_type: String,
) -> Result<CharacterLocalised, AppError> {
    let path = "json/message/vn/character.json";
    load_localization(guild_id, path, db_type).await
}

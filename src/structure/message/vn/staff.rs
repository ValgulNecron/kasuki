use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::structure::message::common::load_localization;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StaffLocalised {
    pub main: String,

    pub aid: String,

    pub gender: String,

    pub lang: String,
}

pub async fn load_localization_staff(
    guild_id: String,
    db_type: String,
) -> Result<StaffLocalised, Box<dyn Error>> {
    let path = "json/message/vn/staff.json";
    load_localization(guild_id, path, db_type).await
}

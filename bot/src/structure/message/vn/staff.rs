use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]

pub struct StaffLocalised {
    pub main: String,

    pub aid: String,

    pub gender: String,

    pub lang: String,
}

use anyhow::Result;

pub async fn load_localization_staff(
    guild_id: String,
    db_config: DbConfig,
) -> Result<StaffLocalised> {

    let path = "json/message/vn/staff.json";

    load_localization(guild_id, path, db_config).await
}

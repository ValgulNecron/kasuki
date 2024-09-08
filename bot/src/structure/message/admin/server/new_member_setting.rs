use std::error::Error;

use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]

pub struct NewMemberSettingLocalised {
    pub title: String,

    pub description: String,
}

pub async fn load_localization_new_member_setting(
    guild_id: String,
    db_config: DbConfig,
) -> Result<NewMemberSettingLocalised, Box<dyn Error>> {

    let path = "json/message/admin/server/new_member_setting.json";

    load_localization(guild_id, path, db_config).await
}

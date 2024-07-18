use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::structure::message::common::load_localization;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewMember {
    pub welcome: String,
}

pub async fn load_localization_new_member(
    guild_id: String,
    db_type: String,
) -> Result<NewMember, Box<dyn Error>> {
    let path = "json/message/new_member.json";
    load_localization(guild_id, path, db_type).await
}

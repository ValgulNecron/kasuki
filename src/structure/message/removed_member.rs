use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RemovedMember {
    pub bye: String,
}
pub async fn load_localization_removed_member(
    guild_id: String,
    db_type: String,
) -> Result<RemovedMember, Box<dyn Error>> {
    let path = "json/message/removed_member.json";
    load_localization(guild_id, path, db_type).await
}

use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct TranscriptLocalised {
    pub title: String,
}

use anyhow::Result;

pub async fn load_localization_transcript(
    guild_id: String,
    db_config: DbConfig,
) -> Result<TranscriptLocalised> {
    let path = "json/message/ai/transcript.json";

    load_localization(guild_id, path, db_config).await
}

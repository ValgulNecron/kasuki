use std::error::Error;

use crate::config::DbConfig;
use serde::{Deserialize, Serialize};

// Importing necessary libraries and modules
use crate::structure::message::common::load_localization;

/// SendActivityLocalised struct represents a send activity's localized data.
/// It contains fields for title and description.
///
/// # Fields
/// * `title`: A string representing the title related data.
/// * `desc`: A string representing the description related data.
#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct SendActivityLocalised {
    pub title: String,
    pub desc: String,
}

use anyhow::{Context, Result};

pub async fn load_localization_send_activity(
    guild_id: String,
    db_config: DbConfig,
) -> Result<SendActivityLocalised> {

    let path = "json/message/anilist_user/send_activity.json";

    load_localization(guild_id, path, db_config).await
}

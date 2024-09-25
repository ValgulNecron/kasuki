use crate::config::DbConfig;
use crate::structure::message::common::load_localization;
use serde::{Deserialize, Serialize};

/// Represents the localized banner data.
///
/// This struct is used to deserialize the JSON data from the localization file.
/// It contains three fields `title`, `no_banner` and `no_banner_title` which are all Strings.
#[derive(Debug, Deserialize, Serialize, Clone)]

pub struct BannerLocalised {
    pub title: String,
    pub no_banner: String,
    pub no_banner_title: String,
}

/// Loads the localized banner data.
///
/// This function reads the localization data from a JSON file, parses it into a HashMap,
/// and then returns the localized banner data for the specified guild.
///
/// # Arguments
///
/// * `guild_id` - The ID of the guild for which to load the localized banner data.
///
/// # Returns
///
/// * `Result<BannerLocalised, AppError>` - The localized banner data,
///   or an error if the file could not be read, the JSON could not be parsed, or the language could not be found.
///
/// # Errors
///
/// This function will return an `AppError` if it encounters any issues while reading or parsing the JSON file.
/// It will also return an `AppError` if the language specified by the `guild_id` is not found in the JSON data.
use anyhow::Result;

pub async fn load_localization_banner(
    guild_id: String,
    db_config: DbConfig,
) -> Result<BannerLocalised> {

    let path = "json/message/user/banner.json";

    load_localization(guild_id, path, db_config).await
}

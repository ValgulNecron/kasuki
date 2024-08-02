use std::error::Error;

use serde::{Deserialize, Serialize};

// Importing necessary libraries and modules
use crate::structure::message::common::load_localization;

/// MediaLocalised struct represents a media's localized data.
/// It contains fields for two titles, a description, and staff text.
///
/// # Struct Fields
/// `field1_title`: A String representing the first title of the media.
/// `field2_title`: A String representing the second title of the media.
/// `desc`: A String representing the description of the media.
/// `staff_text`: A String representing the staff text of the media.

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MediaLocalised {
    pub genre: String,

    pub tag: String,

    pub staffs: String,

    pub characters: String,

    pub format: String,

    pub source: String,

    pub start_date: String,

    pub end_date: String,

    pub fav: String,

    pub duration: String,
    pub chapter: String,

    pub minutes: String,
}

/// This function loads the localization data for a media item.
/// It takes a guild_id as input and returns a Result containing MediaLocalised data or an AppError.
/// The function reads a JSON file, parses it into a HashMap, and then retrieves the data based on the guild's language.
///
/// # Arguments
///
/// * `guild_id` - A string slice that holds the guild id.
///
/// # Returns
///
/// * `Result<MediaLocalised, AppError>` - A Result type which is either MediaLocalised data or an AppError.
///
/// # Errors
///
/// This function will return an error if the JSON file cannot be read, the JSON cannot be parsed, or the language is not found.
pub async fn load_localization_media(
    guild_id: String,
    db_type: String,
) -> Result<MediaLocalised, Box<dyn Error>> {
    let path = "json/message/anilist_user/media.json";
    load_localization(guild_id, path, db_type).await
}

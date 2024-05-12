
use serde::{Deserialize, Serialize};

// Importing necessary libraries and modules
use crate::helper::error_management::error_enum::{AppError};
use crate::structure::message::common::load_localization;

/// MediaLocalised struct represents a media's localized data.
/// It contains fields for two titles, a description, and staff text.
///
/// # Struct Fields
/// `field1_title`: A String representing the first title of the media.
/// `field2_title`: A String representing the second title of the media.
/// `desc`: A String representing the description of the media.
/// `staff_text`: A String representing the staff text of the media.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MediaLocalised {
    pub field1_title: String,
    pub field2_title: String,
    pub desc: String,
    pub staff_text: String,
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
pub async fn load_localization_media(guild_id: String) -> Result<MediaLocalised, AppError> {
    let path = "json/message/anilist_user/media.json";
    load_localization(guild_id, path).await

}

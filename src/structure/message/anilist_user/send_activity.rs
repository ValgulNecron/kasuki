use serde::{Deserialize, Serialize};

// Importing necessary libraries and modules
use crate::helper::error_management::error_enum::AppError;
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

/// This function loads the localization data for a send activity.
/// It takes a guild_id as input and returns a Result containing SendActivityLocalised data or an AppError.
/// The function reads a JSON file, parses it into a HashMap, and then retrieves the data based on the guild's language.
///
/// # Arguments
///
/// * `guild_id`: A string representing the guild id.
///
/// # Returns
///
/// * `Result<SendActivityLocalised, AppError>`: A Result containing SendActivityLocalised data or an AppError.
pub async fn load_localization_send_activity(
    guild_id: String,
    db_type: &str,
) -> Result<SendActivityLocalised, AppError> {
    let path = "json/message/anilist_user/send_activity.json";
    load_localization(guild_id, path, db_type).await
}

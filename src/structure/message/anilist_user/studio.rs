
use serde::{Deserialize, Serialize};

// Importing necessary libraries and modules
use crate::helper::error_management::error_enum::{AppError};
use crate::structure::message::common::load_localization;

/// StudioLocalised struct represents a studio's localized data.
/// It contains a field for description.
///
/// # Fields
/// * `desc`: A string representing the description related data.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StudioLocalised {
    pub desc: String,
}

/// This function loads the localization data for a studio.
/// It takes a guild_id as input and returns a Result containing StudioLocalised data or an AppError.
/// The function reads a JSON file, parses it into a HashMap, and then retrieves the data based on the guild's language.
///
/// # Arguments
///
/// * `guild_id`: A string representing the guild id.
///
/// # Returns
///
/// * `Result<StudioLocalised, AppError>`: A Result containing StudioLocalised data or an AppError.
pub async fn load_localization_studio(guild_id: String) -> Result<StudioLocalised, AppError> {
    let path = "json/message/anilist_user/studio.json";
    load_localization(guild_id, path).await

}

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// Importing necessary libraries and modules
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::get_guild_lang::get_guild_language;
use crate::helper::read_file::read_file_as_string;
use crate::structure::message::common::load_localization;

/// UserLocalised struct represents a user's localized data.
/// It contains fields for manga, anime, week, day, hour, minute, weeks, days, hours, and minutes.
///
/// # Fields
/// * `manga`: A string representing the manga related data.
/// * `anime`: A string representing the anime related data.
/// * `week`: A string representing the week related data.
/// * `day`: A string representing the day related data.
/// * `hour`: A string representing the hour related data.
/// * `minute`: A string representing the minute related data.
/// * `weeks`: A string representing the weeks related data.
/// * `days`: A string representing the days related data.
/// * `hours`: A string representing the hours related data.
/// * `minutes`: A string representing the minutes related data.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserLocalised {
    pub manga: String,
    pub anime: String,
    pub week: String,
    pub day: String,
    pub hour: String,
    pub minute: String,
    pub weeks: String,
    pub days: String,
    pub hours: String,
    pub minutes: String,
}

/// This function loads the localization data for a user.
/// It takes a guild_id as input and returns a Result containing UserLocalised data or an AppError.
/// The function reads a JSON file, parses it into a HashMap, and then retrieves the data based on the guild's language.
///
/// # Arguments
///
/// * `guild_id`: A string representing the guild id.
///
/// # Returns
///
/// * `Result<UserLocalised, AppError>`: A Result containing UserLocalised data or an AppError.
pub async fn load_localization_user(guild_id: String) -> Result<UserLocalised, AppError> {
    let path = "json/message/anilist_user/user.json";
    load_localization(guild_id, path).await

}

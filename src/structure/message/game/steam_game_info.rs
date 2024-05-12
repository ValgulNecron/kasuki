// Importing necessary libraries and modules

use serde::{Deserialize, Serialize};

use crate::helper::error_management::error_enum::{AppError};
use crate::structure::message::common::load_localization;

/// `SteamGameInfoLocalised` is a struct that represents a Steam game's localized data.
/// It contains several fields which are all Strings.
///
/// # Struct Fields
/// `field1` to `field7`, `free`, `coming_soon`, `tba`: Strings representing different pieces of information about the Steam game.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SteamGameInfoLocalised {
    pub field1: String,
    pub field2: String,
    pub field3: String,
    pub field4: String,
    pub field5: String,
    pub field6: String,
    pub field7: String,
    pub free: String,
    pub coming_soon: String,
    pub tba: String,
}

/// `load_localization_steam_game_info` is an asynchronous function that loads the localized data for a Steam game.
/// It takes a `guild_id` as a parameter which is a String.
/// It returns a Result which is either a `SteamGameInfoLocalised` struct or an `AppError`.
///
/// # Arguments
///
/// * `guild_id` - A string slice that holds the guild id.
///
/// # Returns
///
/// * `Result<SteamGameInfoLocalised, AppError>` - A Result type which is either SteamGameInfoLocalised data or an AppError.
///
/// # Errors
///
/// This function will return an `AppError` if it encounters any issues while reading or parsing the JSON file.
/// It will also return an `AppError` if the language specified by the `guild_id` is not found in the JSON data.
pub async fn load_localization_steam_game_info(
    guild_id: String,
) -> Result<SteamGameInfoLocalised, AppError> {
    let path = "json/message/game/steam_game_info.json";
    load_localization(guild_id, path).await

}

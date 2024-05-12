use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::from_str;

use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::get_guild_lang::get_guild_language;
use crate::helper::read_file::read_file_as_string;
use crate::structure::message::common::load_localization;

/// Represents the localized messages for a new member.
///
/// This struct is used to deserialize the JSON data from the localization file.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NewMemberLocalised {
    /// The welcome message for the new member.
    pub welcome: String,
}

/// Loads the localized messages for a new member.
///
/// This function reads the localization data from a JSON file, parses it into a HashMap,
/// and then returns the localized messages for the specified guild.
///
/// # Arguments
///
/// * `guild_id` - The ID of the guild for which to load the localized messages.
///
/// # Returns
///
/// * `Result<NewMemberLocalised, AppError>` - The localized messages for the new member,
///   or an error if the file could not be read, the JSON could not be parsed, or the language could not be found.
pub async fn load_localization_new_member(
    guild_id: String,
) -> Result<NewMemberLocalised, AppError> {
    let path = "json/message/new_member.json";
    load_localization(guild_id, path).await

}

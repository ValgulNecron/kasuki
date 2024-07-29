use std::collections::HashMap;
use std::error::Error;

use crate::helper::error_management::error_enum::UnknownResponseError;
use crate::helper::get_guild_lang::get_guild_language;
use crate::helper::read_file::read_file_as_string;

pub async fn load_localization<'a, T: serde::Deserialize<'a> + Clone>(
    guild_id: String,
    path: &str,
    db_type: String,
) -> Result<T, Box<dyn Error>> {
    let json_content = read_file_as_string(path)?;
    let json: &'a str = Box::leak(json_content.into_boxed_str());
    // Parse the JSON data into a HashMap and handle any potential errors
    let json_data: HashMap<String, T> =
        serde_json::from_str(json).map_err(|e| UnknownResponseError::Json(format!("{:#?}", e)))?;

    // Get the language choice for the guild
    let lang_choice = get_guild_language(guild_id, db_type).await;

    // Retrieve the localized data for the add activity based on the language choice
    Ok(json_data
        .get(lang_choice.as_str())
        .cloned()
        .unwrap_or(match json_data.get("en") {
            Some(data) => data.clone(),
            None => Err(UnknownResponseError::Json(String::from(
                "Your set language or english is not found please report this issue.",
            )))?,
        }))
}

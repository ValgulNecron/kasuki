use crate::config::DbConfig;
use crate::helper::get_guild_lang::get_guild_language;
use crate::helper::read_file::read_file_as_string;
use anyhow::{Context, Result};
use std::collections::HashMap;

pub async fn load_localization<'a, T: serde::Deserialize<'a> + Clone>(
	guild_id: String, path: &str, db_config: DbConfig,
) -> Result<T> {
	let json_content = read_file_as_string(path).context(format!("Failed to read file: {}", path))?;

	let json: &'a str = Box::leak(json_content.into_boxed_str());

	// Parse the JSON data into a HashMap and handle any potential errors
	let json_data: HashMap<String, T> =
		serde_json::from_str(json).context("Failed to parse JSON data")?;

	// Get the language choice for the guild
	let lang_choice = get_guild_language(guild_id, db_config).await;

	// Retrieve the localized data for the add activity based on the language choice
	Ok(json_data.get(lang_choice.as_str()).cloned().unwrap_or(
		json_data
			.get("en")
			.cloned()
			.context("Failed to get English localization")?,
	))
}

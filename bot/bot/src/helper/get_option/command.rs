use std::collections::HashMap;

use serenity::all::{CommandInteraction, GenericChannelId, UserId};
use small_fixed_array::FixedString;

pub fn get_option_map_string(interaction: &CommandInteraction) -> HashMap<FixedString, String> {
	let mut map = HashMap::new();

	for option in &interaction.data.options {
		let name = option.name.clone();

		let value = match option.value.as_str() {
			Some(value) => value.to_string(),
			None => continue,
		};

		map.insert(name, value);
	}

	map
}

#[allow(dead_code)]
pub fn get_option_map_integer(interaction: &CommandInteraction) -> HashMap<FixedString, i64> {
	let mut map = HashMap::new();

	for option in &interaction.data.options {
		let name = option.name.clone();

		let value = match option.value.as_i64() {
			Some(value) => value,
			None => continue,
		};

		map.insert(name, value);
	}

	map
}

pub fn get_option_map_boolean(interaction: &CommandInteraction) -> HashMap<FixedString, bool> {
	let mut map = HashMap::new();

	for option in &interaction.data.options {
		let name = option.name.clone();

		let value = match option.value.as_bool() {
			Some(value) => value,
			None => continue,
		};

		map.insert(name, value);
	}

	map
}

pub fn get_option_map_user(interaction: &CommandInteraction) -> HashMap<FixedString, UserId> {
	let mut map = HashMap::new();

	for option in &interaction.data.options {
		let name = option.name.clone();

		let value = match option.value.as_user_id() {
			Some(value) => value,
			None => continue,
		};

		map.insert(name, value);
	}

	map
}

#[allow(dead_code)]
pub fn get_option_map_channel(
	interaction: &CommandInteraction,
) -> HashMap<FixedString, GenericChannelId> {
	let mut map = HashMap::new();

	for option in &interaction.data.options {
		let name = option.name.clone();

		let value = match option.value.as_channel_id() {
			Some(value) => value,
			None => continue,
		};

		map.insert(name, value);
	}

	map
}

/// Extract a single string option by name. Returns an empty string if not present.
pub fn get_string(interaction: &CommandInteraction, name: &str) -> String {
	interaction
		.data
		.options
		.iter()
		.find(|o| o.name.as_str() == name)
		.and_then(|o| o.value.as_str())
		.unwrap_or_default()
		.to_string()
}

/// Extract an optional string option by name.
pub fn get_string_opt(interaction: &CommandInteraction, name: &str) -> Option<String> {
	interaction
		.data
		.options
		.iter()
		.find(|o| o.name.as_str() == name)
		.and_then(|o| o.value.as_str())
		.map(|s| s.to_string())
}

/// Extract an optional i64 integer option by name.
pub fn get_i64(interaction: &CommandInteraction, name: &str) -> Option<i64> {
	interaction
		.data
		.options
		.iter()
		.find(|o| o.name.as_str() == name)
		.and_then(|o| o.value.as_i64())
}

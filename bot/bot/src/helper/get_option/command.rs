use std::collections::HashMap;

use serenity::all::{CommandInteraction, UserId};
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


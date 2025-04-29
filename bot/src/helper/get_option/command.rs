use std::collections::HashMap;

use serenity::all::{AttachmentId, CommandInteraction, GenericChannelId, RoleId, UserId};
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

pub fn get_option_map_role(interaction: &CommandInteraction) -> HashMap<FixedString, RoleId> {
	let mut map = HashMap::new();

	for option in &interaction.data.options {
		let name = option.name.clone();

		let value = match option.value.as_role_id() {
			Some(value) => value,
			None => continue,
		};

		map.insert(name, value);
	}

	map
}

pub fn get_option_map_number(interaction: &CommandInteraction) -> HashMap<FixedString, f64> {
	let mut map = HashMap::new();

	for option in &interaction.data.options {
		let name = option.name.clone();

		let value = match option.value.as_f64() {
			Some(value) => value,
			None => continue,
		};

		map.insert(name, value);
	}

	map
}

pub fn get_option_map_attachment(
	interaction: &CommandInteraction,
) -> HashMap<FixedString, AttachmentId> {
	let mut map = HashMap::new();

	for option in &interaction.data.options {
		let name = option.name.clone();

		let value = match option.value.as_attachment_id() {
			Some(value) => value,
			None => continue,
		};

		map.insert(name, value);
	}

	map
}

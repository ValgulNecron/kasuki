use std::collections::HashMap;

use serenity::all::{Attachment, ChannelId, CommandInteraction, GenericChannelId, ResolvedOption, ResolvedValue, RoleId, UserId};

pub fn get_subcommand(interaction: &CommandInteraction) -> Option<ResolvedOption<'_>> {
	let subcommand_group_value = interaction.data.options().first()?.clone();

	if let ResolvedValue::SubCommandGroup(subcommand_group_options) = subcommand_group_value.value {
		return Some(subcommand_group_options.first()?.clone());
	};

	None
}

pub fn get_option_map_string_subcommand_group(
	interaction: &CommandInteraction,
) -> HashMap<String, String> {
	let mut map = HashMap::new();

	let binding = interaction.data.options();

	let subcommand_group = &binding.first().unwrap().value;

	if let ResolvedValue::SubCommandGroup(subcommand_group_options) = subcommand_group {
		for option in subcommand_group_options {
			if let ResolvedValue::SubCommand(subcommand_options) = &option.value {
				for option2 in subcommand_options {
					let name = option2.name.to_string();

					let value = match option2.value {
						ResolvedValue::String(a) => a.to_string(),
						_ => String::new(),
					};

					map.insert(name, value);
				}
			}
		}
	}

	map
}

pub fn get_option_map_integer_subcommand_group(
	interaction: &CommandInteraction,
) -> HashMap<String, i64> {
	let mut map = HashMap::new();

	let binding = interaction.data.options();

	let subcommand_group = &binding.first().unwrap().value;

	if let ResolvedValue::SubCommandGroup(subcommand_group_options) = subcommand_group {
		for option in subcommand_group_options {
			if let ResolvedValue::SubCommand(subcommand_options) = &option.value {
				for option2 in subcommand_options {
					let name = option2.name.to_string();

					let value = match option2.value {
						ResolvedValue::Integer(a) => a,
						_ => 0,
					};

					map.insert(name, value);
				}
			}
		}
	}

	map
}

pub fn get_option_map_boolean_subcommand_group(
	interaction: &CommandInteraction,
) -> HashMap<String, bool> {
	let mut map = HashMap::new();

	let binding = interaction.data.options();

	let subcommand_group = &binding.first().unwrap().value;

	if let ResolvedValue::SubCommandGroup(subcommand_group_options) = subcommand_group {
		for option in subcommand_group_options {
			if let ResolvedValue::SubCommand(subcommand_options) = &option.value {
				for option2 in subcommand_options {
					let name = option2.name.to_string();

					let value = match option2.value {
						ResolvedValue::Boolean(a) => a,
						_ => false,
					};

					map.insert(name, value);
				}
			}
		}
	}

	map
}

pub fn get_option_map_user_subcommand_group(
	interaction: &CommandInteraction,
) -> HashMap<String, UserId> {
	let mut map = HashMap::new();

	let binding = interaction.data.options();

	let subcommand = &binding.first().unwrap().value;

	if let ResolvedValue::SubCommandGroup(op) = subcommand {
		for option in op {
			if let ResolvedValue::SubCommand(op2) = &option.value {
				for option2 in op2 {
					let name = option2.name.to_string();

					let value = match option2.value {
						ResolvedValue::User(user, _partial_member) => user.id,
						_ => UserId::new(1),
					};

					map.insert(name, value);
				}
			}
		}
	}

	map
}

pub fn get_option_map_channel_subcommand_group(
	interaction: &CommandInteraction,
) -> HashMap<String, GenericChannelId> {
	let mut map = HashMap::new();

	let binding = interaction.data.options();

	let subcommand = &binding.first().unwrap().value;

	if let ResolvedValue::SubCommandGroup(op) = subcommand {
		for option in op {
			if let ResolvedValue::SubCommand(op2) = &option.value {
				for option2 in op2 {
					let name = option2.name.to_string();

					let value = match option2.value {
						ResolvedValue::Channel(a) => a.id(),
						_ => GenericChannelId::from(ChannelId::new(1)),
					};

					map.insert(name, value);
				}
			}
		}
	}

	map
}

pub fn get_option_map_role_subcommand_group(
	interaction: &CommandInteraction,
) -> HashMap<String, RoleId> {
	let mut map = HashMap::new();

	let binding = interaction.data.options();

	let subcommand = &binding.first().unwrap().value;

	if let ResolvedValue::SubCommandGroup(op) = subcommand {
		for option in op {
			if let ResolvedValue::SubCommand(op2) = &option.value {
				for option2 in op2 {
					let name = option2.name.to_string();

					let value = match option2.value {
						ResolvedValue::Role(a) => a.id,
						_ => RoleId::new(1),
					};

					map.insert(name, value);
				}
			}
		}
	}

	map
}

pub fn get_option_map_number_subcommand_group(
	interaction: &CommandInteraction,
) -> HashMap<String, f64> {
	let mut map = HashMap::new();

	let binding = interaction.data.options();

	let subcommand = &binding.first().unwrap().value;

	if let ResolvedValue::SubCommandGroup(op) = subcommand {
		for option in op {
			if let ResolvedValue::SubCommand(op2) = &option.value {
				for option2 in op2 {
					let name = option2.name.to_string();

					let value = match option2.value {
						ResolvedValue::Number(a) => a,
						_ => 0.0,
					};

					map.insert(name, value);
				}
			}
		}
	}

	map
}

pub fn get_option_map_attachment_subcommand_group(
	interaction: &CommandInteraction,
) -> HashMap<String, Attachment> {
	let mut map = HashMap::new();

	let binding = interaction.data.options();

	let subcommand = &binding.first().unwrap().value;

	if let ResolvedValue::SubCommandGroup(op) = subcommand {
		for option in op {
			if let ResolvedValue::SubCommand(op2) = &option.value {
				for option2 in op2 {
					let name = option2.name.to_string();

					let value = match option2.value {
						ResolvedValue::Attachment(a) => a.clone(),
						_ => continue,
					};

					map.insert(name, value);
				}
			}
		}
	}

	map
}

pub fn get_option_map_string_autocomplete_subcommand_group(
	interaction: &CommandInteraction,
) -> HashMap<String, String> {
	let mut map = HashMap::new();

	let binding = interaction.data.options();

	let subcommand_group = &binding.first().unwrap().value;

	if let ResolvedValue::SubCommandGroup(subcommand_group_options) = subcommand_group {
		for option in subcommand_group_options {
			if let ResolvedValue::SubCommand(subcommand_options) = &option.value {
				for option2 in subcommand_options {
					let name = option2.name.to_string();

					let value = match option2.value {
						ResolvedValue::Autocomplete { kind: _, value } => value.to_string(),
						_ => String::new(),
					};

					map.insert(name, value);
				}
			}
		}
	}

	map
}

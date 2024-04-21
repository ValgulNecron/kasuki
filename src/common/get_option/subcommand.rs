use std::collections::HashMap;

use serenity::all::{Attachment, ChannelId, CommandInteraction, ResolvedValue, RoleId, UserId};

pub fn get_option_map_string_subcommand(
    interaction: &CommandInteraction,
) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let binding = interaction.data.options();
    let subcommand = &binding.first().unwrap().value;
    if let ResolvedValue::SubCommand(op) = subcommand {
        for option in op {
            let name = option.name.to_string();
            let value = match option.value {
                ResolvedValue::String(a) => a.to_string(),
                _ => String::new(),
            };
            map.insert(name, value);
        }
    }
    map
}

pub fn get_option_map_integer_subcommand(interaction: &CommandInteraction) -> HashMap<String, i64> {
    let mut map = HashMap::new();
    let binding = interaction.data.options();
    let subcommand = &binding.first().unwrap().value;
    if let ResolvedValue::SubCommand(op) = subcommand {
        for option in op {
            let name = option.name.to_string();
            let value = match option.value {
                ResolvedValue::Integer(a) => a,
                _ => 0,
            };
            map.insert(name, value);
        }
    }
    map
}

pub fn get_option_map_boolean_subcommand(
    interaction: &CommandInteraction,
) -> HashMap<String, bool> {
    let mut map = HashMap::new();
    let binding = interaction.data.options();
    let subcommand = &binding.first().unwrap().value;
    if let ResolvedValue::SubCommand(op) = subcommand {
        for option in op {
            let name = option.name.to_string();
            let value = match option.value {
                ResolvedValue::Boolean(a) => a,
                _ => false,
            };
            map.insert(name, value);
        }
    }
    map
}

pub fn get_option_map_user_subcommand(interaction: &CommandInteraction) -> HashMap<String, UserId> {
    let mut map = HashMap::new();
    let binding = interaction.data.options();
    let subcommand = &binding.first().unwrap().value;
    if let ResolvedValue::SubCommand(op) = subcommand {
        for option in op {
            let name = option.name.to_string();
            let value = match option.value {
                ResolvedValue::User(user, _partial_member) => user.id,
                _ => UserId::new(1),
            };
            map.insert(name, value);
        }
    }
    map
}

pub fn get_option_map_channel_subcommand(
    interaction: &CommandInteraction,
) -> HashMap<String, ChannelId> {
    let mut map = HashMap::new();
    let binding = interaction.data.options();
    let subcommand = &binding.first().unwrap().value;
    if let ResolvedValue::SubCommand(op) = subcommand {
        for option in op {
            let name = option.name.to_string();
            let value = match option.value {
                ResolvedValue::Channel(a) => a.id,
                _ => ChannelId::new(1),
            };
            map.insert(name, value);
        }
    }
    map
}

pub fn get_option_map_role_subcommand(interaction: &CommandInteraction) -> HashMap<String, RoleId> {
    let mut map = HashMap::new();
    let binding = interaction.data.options();
    let subcommand = &binding.first().unwrap().value;
    if let ResolvedValue::SubCommand(op) = subcommand {
        for option in op {
            let name = option.name.to_string();
            let value = match option.value {
                ResolvedValue::Role(a) => a.id,
                _ => RoleId::new(1),
            };
            map.insert(name, value);
        }
    }
    map
}

pub fn get_option_map_number_subcommand(interaction: &CommandInteraction) -> HashMap<String, f64> {
    let mut map = HashMap::new();
    let binding = interaction.data.options();
    let subcommand = &binding.first().unwrap().value;
    if let ResolvedValue::SubCommand(op) = subcommand {
        for option in op {
            let name = option.name.to_string();
            let value = match option.value {
                ResolvedValue::Number(a) => a,
                _ => 0.0,
            };
            map.insert(name, value);
        }
    }
    map
}

pub fn get_option_map_attachment_subcommand(
    interaction: &CommandInteraction,
) -> HashMap<String, Attachment> {
    let mut map = HashMap::new();
    let binding = interaction.data.options();
    let subcommand = &binding.first().unwrap().value;
    if let ResolvedValue::SubCommand(op) = subcommand {
        for option in op {
            let name = option.name.to_string();
            let value = match option.value {
                ResolvedValue::Attachment(a) => a.clone(),
                _ => continue,
            };
            map.insert(name, value);
        }
    }
    map
}

pub fn get_option_map_string_autocomplete_subcommand(
    interaction: &CommandInteraction,
) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let binding = interaction.data.options();
    let subcommand = &binding.first().unwrap().value;
    if let ResolvedValue::SubCommand(op) = subcommand {
        for option in op {
            let name = option.name.to_string();
            let value = match &option.value {
                ResolvedValue::Autocomplete { kind: _, value } => value.to_string(),
                // Handle other types as needed
                _ => String::new(),
            };
            map.insert(name, value);
        }
    }

    map
}

use std::collections::HashMap;

use serenity::all::{
    CommandInteraction, ResolvedOption, ResolvedValue,
};

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

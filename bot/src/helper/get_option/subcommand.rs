use std::collections::HashMap;

use serenity::all::{Attachment, ChannelId, CommandInteraction, ResolvedValue, RoleId, UserId};

/// Retrieves the string options from the subcommand in the command interaction and returns them as a HashMap.
///
/// This function first retrieves the subcommand from the command interaction.
/// It then iterates over the options in the subcommand and extracts the string options.
/// These options are inserted into a HashMap with the option name as the key and the option value as the value.
///
/// # Arguments
///
/// * `interaction` - The command interaction from which to extract the options.
///
/// # Returns
///
/// A `HashMap` where the keys are the option names and the values are the option values.

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

/// Retrieves the integer options from the subcommand in the command interaction and returns them as a HashMap.
///
/// This function first retrieves the subcommand from the command interaction.
/// It then iterates over the options in the subcommand and extracts the integer options.
/// These options are inserted into a HashMap with the option name as the key and the option value as the value.
///
/// # Arguments
///
/// * `interaction` - The command interaction from which to extract the options.
///
/// # Returns
///
/// A `HashMap` where the keys are the option names and the values are the option values.

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

/// Retrieves the boolean options from the subcommand in the command interaction and returns them as a HashMap.
///
/// This function first retrieves the subcommand from the command interaction.
/// It then iterates over the options in the subcommand and extracts the boolean options.
/// These options are inserted into a HashMap with the option name as the key and the option value as the value.
///
/// # Arguments
///
/// * `interaction` - The command interaction from which to extract the options.
///
/// # Returns
///
/// A `HashMap` where the keys are the option names and the values are the option values.

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

/// Retrieves the user options from the subcommand in the command interaction and returns them as a HashMap.
///
/// This function first retrieves the subcommand from the command interaction.
/// It then iterates over the options in the subcommand and extracts the user options.
/// These options are inserted into a HashMap with the option name as the key and the option value as the value.
///
/// # Arguments
///
/// * `interaction` - The command interaction from which to extract the options.
///
/// # Returns
///
/// A `HashMap` where the keys are the option names and the values are the option values.

pub fn get_option_map_user_subcommand(interaction: &CommandInteraction) -> HashMap<String, UserId> {
    let mut map = HashMap::new();

    let binding = interaction.data.options();

    let subcommand = &binding.first().unwrap().value;

    if let ResolvedValue::SubCommand(op) = subcommand {
        for option in op {
            let name = option.name.to_string();

            let value = match &option.value {
                ResolvedValue::User(user, _partial_member) => user.id,
                _ => UserId::new(1),
            };

            map.insert(name, value);
        }
    }

    map
}

/// Retrieves the channel options from the subcommand in the command interaction and returns them as a HashMap.
///
/// This function first retrieves the subcommand from the command interaction.
/// It then iterates over the options in the subcommand and extracts the channel options.
/// These options are inserted into a HashMap with the option name as the key and the option value as the value.
///
/// # Arguments
///
/// * `interaction` - The command interaction from which to extract the options.
///
/// # Returns
///
/// A `HashMap` where the keys are the option names and the values are the option values.

pub fn get_option_map_channel_subcommand(
    interaction: &CommandInteraction,
) -> HashMap<String, ChannelId> {
    let mut map = HashMap::new();

    let binding = interaction.data.options();

    let subcommand = &binding.first().unwrap().value;

    if let ResolvedValue::SubCommand(op) = subcommand {
        for option in op {
            let name = option.name.to_string();

            let value = match &option.value {
                ResolvedValue::Channel(a) => a.id,
                _ => ChannelId::new(1),
            };

            map.insert(name, value);
        }
    }

    map
}

/// Retrieves the role options from the subcommand in the command interaction and returns them as a HashMap.
///
/// This function first retrieves the subcommand from the command interaction.
/// It then iterates over the options in the subcommand and extracts the role options.
/// These options are inserted into a HashMap with the option name as the key and the option value as the value.
///
/// # Arguments
///
/// * `interaction` - The command interaction from which to extract the options.
///
/// # Returns
///
/// A `HashMap` where the keys are the option names and the values are the option values.

pub fn get_option_map_role_subcommand(interaction: &CommandInteraction) -> HashMap<String, RoleId> {
    let mut map = HashMap::new();

    let binding = interaction.data.options();

    let subcommand = &binding.first().unwrap().value;

    if let ResolvedValue::SubCommand(op) = subcommand {
        for option in op {
            let name = option.name.to_string();

            let value = match &option.value {
                ResolvedValue::Role(a) => a.id,
                _ => RoleId::new(1),
            };

            map.insert(name, value);
        }
    }

    map
}

/// Retrieves the number options from the subcommand in the command interaction and returns them as a HashMap.
///
/// This function first retrieves the subcommand from the command interaction.
/// It then iterates over the options in the subcommand and extracts the number options.
/// These options are inserted into a HashMap with the option name as the key and the option value as the value.
///
/// # Arguments
///
/// * `interaction` - The command interaction from which to extract the options.
///
/// # Returns
///
/// A `HashMap` where the keys are the option names and the values are the option values.

pub fn get_option_map_number_subcommand(interaction: &CommandInteraction) -> HashMap<String, f64> {
    let mut map = HashMap::new();

    let binding = interaction.data.options();

    let subcommand = &binding.first().unwrap().value;

    if let ResolvedValue::SubCommand(op) = subcommand {
        for option in op {
            let name = option.name.to_string();

            let value = match &option.value {
                ResolvedValue::Number(a) => *a,
                _ => 0.0,
            };

            map.insert(name, value);
        }
    }

    map
}

/// Retrieves the attachment options from the subcommand in the command interaction and returns them as a HashMap.
///
/// This function first retrieves the subcommand from the command interaction.
/// It then iterates over the options in the subcommand and extracts the attachment options.
/// These options are inserted into a HashMap with the option name as the key and the option value as the value.
///
/// # Arguments
///
/// * `interaction` - The command interaction from which to extract the options.
///
/// # Returns
///
/// A `HashMap` where the keys are the option names and the values are the option values.

pub fn get_option_map_attachment_subcommand(
    interaction: &CommandInteraction,
) -> HashMap<String, Attachment> {
    let mut map = HashMap::new();

    let binding = interaction.data.options();

    let subcommand = &binding.first().unwrap().value;

    if let ResolvedValue::SubCommand(op) = subcommand {
        for option in op {
            let name = option.name.to_string();

            let value = match &option.value {
                ResolvedValue::Attachment(a) => {
                    let att = *a;

                    att.clone()
                }
                _ => continue,
            };

            map.insert(name, value);
        }
    }

    map
}

/// Retrieves the string options from the autocomplete subcommand in the command interaction and returns them as a HashMap.
///
/// This function first retrieves the subcommand from the command interaction.
/// It then iterates over the options in the subcommand and extracts the string options from the autocomplete subcommand.
/// These options are inserted into a HashMap with the option name as the key and the option value as the value.
///
/// # Arguments
///
/// * `interaction` - The command interaction from which to extract the options.
///
/// # Returns
///
/// A `HashMap` where the keys are the option names and the values are the option values.

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

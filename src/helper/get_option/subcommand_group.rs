use std::collections::HashMap;

use serenity::all::{
    Attachment, ChannelId, CommandInteraction, ResolvedOption, ResolvedValue, RoleId, UserId,
};

/// Retrieves the first subcommand group from the command interaction.
///
/// This function first retrieves the options from the command interaction.
/// It then checks if the first option is a subcommand group.
/// If it is, it returns the first option of the subcommand group.
///
/// # Arguments
///
/// * `interaction` - The command interaction from which to extract the subcommand group.
///
/// # Returns
///
/// An `Option` that contains the first `ResolvedOption` of the subcommand group if it exists, or `None` if it doesn't.
pub fn get_subcommand(interaction: &CommandInteraction) -> Option<ResolvedOption<'_>> {
    let subcommand_group_value = interaction.data.options().first().unwrap().clone();
    if let ResolvedValue::SubCommandGroup(subcommand_group_options) = subcommand_group_value.value {
        return Some(subcommand_group_options.first().unwrap().clone());
    };
    None
}

/// Retrieves the string options from the subcommand group in the command interaction and returns them as a HashMap.
///
/// This function first retrieves the subcommand group from the command interaction.
/// It then iterates over the options in the subcommand group and extracts the string options.
/// These options are inserted into a HashMap with the option name as the key and the option value as the value.
///
/// # Arguments
///
/// * `interaction` - The command interaction from which to extract the options.
///
/// # Returns
///
/// A `HashMap` where the keys are the option names and the values are the option values.
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

/// Retrieves the integer options from the subcommand group in the command interaction and returns them as a HashMap.
///
/// This function first retrieves the subcommand group from the command interaction.
/// It then iterates over the options in the subcommand group and extracts the integer options.
/// These options are inserted into a HashMap with the option name as the key and the option value as the value.
///
/// # Arguments
///
/// * `interaction` - The command interaction from which to extract the options.
///
/// # Returns
///
/// A `HashMap` where the keys are the option names and the values are the option values.
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

/// Retrieves the boolean options from the subcommand group in the command interaction and returns them as a HashMap.
///
/// This function first retrieves the subcommand group from the command interaction.
/// It then iterates over the options in the subcommand group and extracts the boolean options.
/// These options are inserted into a HashMap with the option name as the key and the option value as the value.
///
/// # Arguments
///
/// * `interaction` - The command interaction from which to extract the options.
///
/// # Returns
///
/// A `HashMap` where the keys are the option names and the values are the option values.
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

/// Retrieves the user options from the subcommand group in the command interaction and returns them as a HashMap.
///
/// This function first retrieves the subcommand group from the command interaction.
/// It then iterates over the options in the subcommand group and extracts the user options.
/// These options are inserted into a HashMap with the option name as the key and the option value as the value.
///
/// # Arguments
///
/// * `interaction` - The command interaction from which to extract the options.
///
/// # Returns
///
/// A `HashMap` where the keys are the option names and the values are the option values.
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

/// Retrieves the channel options from the subcommand group in the command interaction and returns them as a HashMap.
///
/// This function first retrieves the subcommand group from the command interaction.
/// It then iterates over the options in the subcommand group and extracts the channel options.
/// These options are inserted into a HashMap with the option name as the key and the option value as the value.
///
/// # Arguments
///
/// * `interaction` - The command interaction from which to extract the options.
///
/// # Returns
///
/// A `HashMap` where the keys are the option names and the values are the option values.
pub fn get_option_map_channel_subcommand_group(
    interaction: &CommandInteraction,
) -> HashMap<String, ChannelId> {
    let mut map = HashMap::new();
    let binding = interaction.data.options();
    let subcommand = &binding.first().unwrap().value;
    if let ResolvedValue::SubCommandGroup(op) = subcommand {
        for option in op {
            if let ResolvedValue::SubCommand(op2) = &option.value {
                for option2 in op2 {
                    let name = option2.name.to_string();
                    let value = match option2.value {
                        ResolvedValue::Channel(a) => a.id,
                        _ => ChannelId::new(1),
                    };
                    map.insert(name, value);
                }
            }
        }
    }
    map
}

/// Retrieves the role options from the subcommand group in the command interaction and returns them as a HashMap.
///
/// This function first retrieves the subcommand group from the command interaction.
/// It then iterates over the options in the subcommand group and extracts the role options.
/// These options are inserted into a HashMap with the option name as the key and the option value as the value.
///
/// # Arguments
///
/// * `interaction` - The command interaction from which to extract the options.
///
/// # Returns
///
/// A `HashMap` where the keys are the option names and the values are the option values.
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

/// Retrieves the number options from the subcommand group in the command interaction and returns them as a HashMap.
///
/// This function first retrieves the subcommand group from the command interaction.
/// It then iterates over the options in the subcommand group and extracts the number options.
/// These options are inserted into a HashMap with the option name as the key and the option value as the value.
///
/// # Arguments
///
/// * `interaction` - The command interaction from which to extract the options.
///
/// # Returns
///
/// A `HashMap` where the keys are the option names and the values are the option values.
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

/// Retrieves the attachment options from the subcommand group in the command interaction and returns them as a HashMap.
///
/// This function first retrieves the subcommand group from the command interaction.
/// It then iterates over the options in the subcommand group and extracts the attachment options.
/// These options are inserted into a HashMap with the option name as the key and the option value as the value.
///
/// # Arguments
///
/// * `interaction` - The command interaction from which to extract the options.
///
/// # Returns
///
/// A `HashMap` where the keys are the option names and the values are the option values.
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

/// Retrieves the string options from the autocomplete subcommand group in the command interaction and returns them as a HashMap.
///
/// This function first retrieves the subcommand group from the command interaction.
/// It then iterates over the options in the subcommand group and extracts the string options from the autocomplete subcommand group.
/// These options are inserted into a HashMap with the option name as the key and the option value as the value.
///
/// # Arguments
///
/// * `interaction` - The command interaction from which to extract the options.
///
/// # Returns
///
/// A `HashMap` where the keys are the option names and the values are the option values.
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

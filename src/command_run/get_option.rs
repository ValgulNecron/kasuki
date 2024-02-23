use std::collections::HashMap;

use serenity::all::{Attachment, CommandInteraction, ResolvedOption, ResolvedValue, UserId};
use tracing::trace;

use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

pub fn get_option_map_string(interaction: &CommandInteraction) -> HashMap<String, String> {
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

pub fn get_option_map_attachment(
    interaction: &CommandInteraction,
) -> HashMap<String, Option<Attachment>> {
    let mut map = HashMap::new();
    for option in &interaction.data.options() {
        let attachment;
        if let ResolvedOption {
            value: ResolvedValue::Attachment(attachment_option),
            ..
        } = option
        {
            let simple = *attachment_option;
            let attach_option = simple.clone();
            attachment = Some(attach_option)
        } else {
            continue;
        }
        let name = option.name.to_string();
        map.insert(name, attachment);
    }

    map
}

pub fn get_the_attachment(
    attachment: Option<&Option<Attachment>>,
) -> Result<&Attachment, AppError> {
    match attachment {
        Some(Some(att)) => Ok(att),
        _ => Err(AppError::new(
            String::from("There is no option"),
            ErrorType::Option,
            ErrorResponseType::Message,
        )),
    }
}

pub fn get_option_map_user(interaction: &CommandInteraction) -> HashMap<String, UserId> {
    let mut map = HashMap::new();
    for option in &interaction.data.options {
        let value = match option.value.as_user_id() {
            Some(user) => user,
            None => continue,
        };
        let name = option.name.clone();
        map.insert(name, value);
    }

    map
}

pub fn get_option_map_bool(interaction: &CommandInteraction) -> HashMap<String, bool> {
    let mut map = HashMap::new();
    for option in &interaction.data.options {
        let value = match option.value.as_bool() {
            Some(value) => value,
            None => continue,
        };
        let name = option.name.clone();
        map.insert(name, value);
    }

    map
}

pub fn get_option_map_string_subcommand(
    interaction: &CommandInteraction,
) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let binding = interaction.data.options();
    let subcommand = &binding.first().unwrap().value;
    match subcommand {
        ResolvedValue::SubCommand(op) => {
            for option in op{
                let name = option.name.to_string();
                let value = match option.value {
                    ResolvedValue::String(a) => a.to_string(),
                    _ => String::new()
                };
                map.insert(name, value);
            }
        }
        _ => {}
    }
    map
}

pub fn get_option_map_attachment_subcommand(
    interaction: &CommandInteraction,
) -> HashMap<String, Option<Attachment>> {
    let mut map = HashMap::new();
    let binding = interaction.data.options();
    let subcommand = &binding.first().unwrap().value;
    match subcommand {
        ResolvedValue::SubCommand(op) => {
            for option in op{
                let name = option.name.to_string();
                let value = match option.value {
                    ResolvedValue::Attachment(a) => Some(a.clone()),
                    _ => None
                };
                map.insert(name, value);
            }
        }
        _ => {}
    }
    map
}
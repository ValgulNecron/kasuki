use crate::error_enum::AppError;
use crate::error_enum::AppError::Error;
use crate::error_enum::CommandError::NoCommandOption;
use serenity::all::{Attachment, CommandInteraction, ResolvedOption, ResolvedValue};
use std::collections::HashMap;

pub fn get_option_map_string(interaction: &CommandInteraction) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for option in &interaction.data.options {
        let value = match option.value.as_str() {
            Some(value) => value.to_string(),
            None => continue,
        };
        let name = option.name.clone();
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
        _ => Err(Error(NoCommandOption(String::from(
            "The command contain no attachment.",
        )))),
    }
}

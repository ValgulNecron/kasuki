use serde::Serialize;
use serde_json;
use serenity::all::CommandOptionType;
use std::fs;
use std::io::Error;

use serde::Deserialize;

#[derive(Debug, Serialize, Deserialize)]
pub struct Arg {
    pub name: String,
    pub desc: String,
    pub required: bool,
    pub autocomplete: bool,
    #[serde(with = "RemoteCommandOptionType", rename = "command_type")]
    pub command_type: RemoteCommandOptionType,
    pub choices: Option<Vec<ArgChoice>>,
    pub localised_args: Option<Vec<LocalisedArg>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArgChoice {
    pub option_choice: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LocalisedArg {
    pub code: String,
    pub name: String,
    pub desc: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Localised {
    pub code: String,
    pub name: String,
    pub desc: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandData {
    pub name: String,
    pub desc: String,
    pub arg_num: u32,
    pub args: Option<Vec<Arg>>,
    pub localised: Option<Vec<Localised>>,
}

pub fn get_commands(directory: &str) -> Result<Vec<CommandData>, Error> {
    let mut commands = Vec::new();

    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().unwrap_or_default() == "json" {
            let json_str = fs::read_to_string(path)?;
            let command: CommandData = serde_json::from_str(&json_str)?;
            commands.push(command);
        }
    }

    Ok(commands)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RemoteCommandOptionType {
    String,
    Integer,
    Boolean,
    User,
    Channel,
    Role,
    Attachment,
    Error,
}

impl From<RemoteCommandOptionType> for CommandOptionType {
    fn from(remote: RemoteCommandOptionType) -> Self {
        match remote {
            RemoteCommandOptionType::String => CommandOptionType::String,
            RemoteCommandOptionType::Integer => CommandOptionType::Integer,
            RemoteCommandOptionType::Boolean => CommandOptionType::Boolean,
            RemoteCommandOptionType::User => CommandOptionType::User,
            RemoteCommandOptionType::Channel => CommandOptionType::Channel,
            RemoteCommandOptionType::Role => CommandOptionType::Role,
            RemoteCommandOptionType::Attachment => CommandOptionType::Attachment,
            _ => CommandOptionType::String,
        }
    }
}

impl From<CommandOptionType> for RemoteCommandOptionType {
    fn from(original: CommandOptionType) -> Self {
        match original {
            CommandOptionType::String => RemoteCommandOptionType::String,
            CommandOptionType::Integer => RemoteCommandOptionType::Integer,
            CommandOptionType::Boolean => RemoteCommandOptionType::Boolean,
            CommandOptionType::User => RemoteCommandOptionType::User,
            CommandOptionType::Channel => RemoteCommandOptionType::Channel,
            CommandOptionType::Role => RemoteCommandOptionType::Role,
            CommandOptionType::Attachment => RemoteCommandOptionType::Attachment,
            _ => RemoteCommandOptionType::String,
        }
    }
}

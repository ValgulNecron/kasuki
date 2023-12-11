use serde::Serialize;
use serde_json;
use serenity::all::{CommandOptionType, Permissions};
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
    pub dm_command: bool,
    pub nsfw: bool,
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

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum RemoteCommandOptionType {
    SubCommand,
    SubCommandGroup,
    String,
    Integer,
    Boolean,
    User,
    Channel,
    Role,
    Mentionable,
    Number,
    Attachment,
    Unknown(u8),
}

impl From<RemoteCommandOptionType> for CommandOptionType {
    fn from(remote: RemoteCommandOptionType) -> Self {
        match remote {
            RemoteCommandOptionType::SubCommand => CommandOptionType::SubCommand,
            RemoteCommandOptionType::SubCommandGroup => CommandOptionType::SubCommandGroup,
            RemoteCommandOptionType::String => CommandOptionType::String,
            RemoteCommandOptionType::Integer => CommandOptionType::Integer,
            RemoteCommandOptionType::Boolean => CommandOptionType::Boolean,
            RemoteCommandOptionType::User => CommandOptionType::User,
            RemoteCommandOptionType::Channel => CommandOptionType::Channel,
            RemoteCommandOptionType::Role => CommandOptionType::Role,
            RemoteCommandOptionType::Mentionable => CommandOptionType::Mentionable,
            RemoteCommandOptionType::Number => CommandOptionType::Number,
            RemoteCommandOptionType::Attachment => CommandOptionType::Attachment,
            RemoteCommandOptionType::Unknown(value) => CommandOptionType::Unknown(value),
            _ => CommandOptionType::String,
        }
    }
}

impl From<CommandOptionType> for RemoteCommandOptionType {
    fn from(original: CommandOptionType) -> Self {
        match original {
            CommandOptionType::SubCommand => RemoteCommandOptionType::SubCommand,
            CommandOptionType::SubCommandGroup => RemoteCommandOptionType::SubCommandGroup,
            CommandOptionType::String => RemoteCommandOptionType::String,
            CommandOptionType::Integer => RemoteCommandOptionType::Integer,
            CommandOptionType::Boolean => RemoteCommandOptionType::Boolean,
            CommandOptionType::User => RemoteCommandOptionType::User,
            CommandOptionType::Channel => RemoteCommandOptionType::Channel,
            CommandOptionType::Role => RemoteCommandOptionType::Role,
            CommandOptionType::Mentionable => RemoteCommandOptionType::Mentionable,
            CommandOptionType::Number => RemoteCommandOptionType::Number,
            CommandOptionType::Attachment => RemoteCommandOptionType::Attachment,
            CommandOptionType::Unknown(value) => RemoteCommandOptionType::Unknown(value),
            _ => RemoteCommandOptionType::String,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum RemotePermissionType {
    CreateInstantInvite,
    KickMembers,
    BanMembers,
    Administrator,
    ManageChannels,
    ManageGuild,
    AddReactions,
    ViewAuditLog,
    PrioritySpeaker,
    Stream,
    ViewChannel,
    SendMessages,
    SendTtsMessages,
    ManageMessages,
    EmbedLinks,
    AttachFiles,
    ReadMessageHistory,
    MentionEveryone,
    UseExternalEmojis,
    ViewGuildInsights,
    Connect,
    Speak,
    MuteMembers,
    DeafenMembers,
    MoveMembers,
    UseVad,
    ChangeNickname,
    ManageNicknames,
    ManageRoles,
    ManageWebhooks,
    ManageGuildExpressions,
    UseApplicationCommands,
    RequestToSpeak,
    ManageEvents,
    ManageThreads,
    CreatePublicThreads,
    CreatePrivateThreads,
    UseExternalStickers,
    SendMessagesInThreads,
    UseEmbeddedActivities,
    ModerateMembers,
    ViewCreatorMonetizationAnalytics,
    UseSoundboard,
    CreateGuildExpressions,
    CreateEvents,
    UseExternalSounds,
    SendVoiceMessages,
    SetVoiceChannelStatus,
    Unknown,
}

impl From<RemotePermissionType> for Permissions {
    fn from(remote: RemotePermissionType) -> Self {
        match remote {
            RemotePermissionType::CreateInstantInvite => Permissions::CREATE_INSTANT_INVITE,
            RemotePermissionType::KickMembers => Permissions::KICK_MEMBERS,
            RemotePermissionType::BanMembers => Permissions::BAN_MEMBERS,
            RemotePermissionType::Administrator => Permissions::ADMINISTRATOR,
            _ => Permissions::ADMINISTRATOR,
        }
    }
}

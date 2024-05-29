use serde::{Deserialize, Serialize};
use serenity::all::{CommandOptionType, Permissions};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommandIntegrationContext {
    pub bot_dm: bool,
    pub private_channel: bool,
    pub guild: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommandInstallationContext {
    pub user: bool,
    pub guild: bool,
}

/// The `Arg` struct represents an argument that a command can accept.
/// It is derived from `Debug`, `Serialize`, `Deserialize`, and `Clone` traits.
///
/// # Fields
///
/// * `name` - The name of the argument as a `String`.
/// * `desc` - The description of the argument as a `String`.
/// * `arg_type` - The type of the argument as a `RemoteCommandOptionType`.
/// * `required` - A `bool` indicating whether the argument is required.
/// * `autocomplete` - A `bool` indicating whether the argument supports autocomplete.
/// * `choices` - An `Option` containing a `Vec` of `Choice` which represents the choices that the argument can accept.
/// * `localised` - An `Option` containing a `Vec` of `Localised` which represents the localised versions of the argument.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Arg {
    pub name: String,
    pub desc: String,
    pub arg_type: RemoteCommandOptionType,
    pub required: bool,
    pub autocomplete: bool,
    pub choices: Option<Vec<Choice>>,
    pub localised: Option<Vec<Localised>>,
}

/// The `Localised` struct represents a localised version of a command or an argument.
/// It is derived from `Debug`, `Serialize`, `Deserialize`, and `Clone` traits.
///
/// # Fields
///
/// * `code` - The language code as a `String`.
/// * `name` - The name in the localised language as a `String`.
/// * `desc` - The description in the localised language as a `String`.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Localised {
    pub code: String,
    pub name: String,
    pub desc: String,
}

/// The `Choice` struct represents a choice that an argument can accept.
/// It is derived from `Debug`, `Serialize`, `Deserialize`, and `Clone` traits.
///
/// # Fields
///
/// * `option_choice` - The choice as a `String`.
/// * `option_choice_localised` - An `Option` containing a `Vec` of `ChoiceLocalised` which represents the localised versions of the choice.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Choice {
    pub option_choice: String,
    pub option_choice_localised: Option<Vec<ChoiceLocalised>>,
}

/// The `ChoiceLocalised` struct represents a localised version of a choice.
/// It is derived from `Debug`, `Serialize`, `Deserialize`, and `Clone` traits.
///
/// # Fields
///
/// * `code` - The language code as a `String`.
/// * `name` - The choice in the localised language as a `String`.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChoiceLocalised {
    pub code: String,
    pub name: String,
}

/// The `DefaultPermission` struct represents a permission required to execute a command.
/// It is derived from `Debug`, `Serialize`, `Deserialize`, and `Clone` traits.
///
/// # Fields
///
/// * `permission` - The permission as a `RemotePermissionType`.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DefaultPermission {
    #[serde(with = "RemotePermissionType")]
    pub permission: RemotePermissionType,
}

/// The `RemoteCommandOptionType` enum represents the type of an argument.
/// It is derived from `Debug`, `Serialize`, `Deserialize`, `Clone`, `Copy`, and `PartialEq` traits.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
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

/// Implementation of the `From` trait for `RemoteCommandOptionType` to `CommandOptionType`.
/// This allows for easy conversion from `RemoteCommandOptionType` to `CommandOptionType`.
///
/// # Arguments
///
/// * `remote` - A `RemoteCommandOptionType` that will be converted to `CommandOptionType`.
///
/// # Returns
///
/// A `CommandOptionType` that corresponds to the given `RemoteCommandOptionType`.
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
        }
    }
}

/// Implementation of the `From` trait for `CommandOptionType` to `RemoteCommandOptionType`.
/// This allows for easy conversion from `CommandOptionType` to `RemoteCommandOptionType`.
///
/// # Arguments
///
/// * `original` - A `CommandOptionType` that will be converted to `RemoteCommandOptionType`.
///
/// # Returns
///
/// A `RemoteCommandOptionType` that corresponds to the given `CommandOptionType`.
/// If the `CommandOptionType` does not have a corresponding `RemoteCommandOptionType`, it defaults to `RemoteCommandOptionType::String`.
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

/// The `RemotePermissionType` enum represents the type of a permission.
/// It is derived from `Debug`, `Serialize`, `Deserialize`, `Clone`, `Copy`, and `PartialEq` traits.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
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

/// Implementation of the `From` trait for `RemotePermissionType` to `Permissions`.
/// This allows for easy conversion from `RemotePermissionType` to `Permissions`.
///
/// # Arguments
///
/// * `remote` - A `RemotePermissionType` that will be converted to `Permissions`.
///
/// # Returns
///
/// A `Permissions` that corresponds to the given `RemotePermissionType`.
/// The conversion is done by matching each variant of `RemotePermissionType` to its corresponding variant in `Permissions`.
/// If the `RemotePermissionType` is `Unknown`, it returns an empty `Permissions`.
impl From<RemotePermissionType> for Permissions {
    fn from(remote: RemotePermissionType) -> Self {
        match remote {
            RemotePermissionType::CreateInstantInvite => Permissions::CREATE_INSTANT_INVITE,
            RemotePermissionType::KickMembers => Permissions::KICK_MEMBERS,
            RemotePermissionType::BanMembers => Permissions::BAN_MEMBERS,
            RemotePermissionType::Administrator => Permissions::ADMINISTRATOR,
            RemotePermissionType::ManageChannels => Permissions::MANAGE_CHANNELS,
            RemotePermissionType::ManageGuild => Permissions::MANAGE_GUILD,
            RemotePermissionType::AddReactions => Permissions::ADD_REACTIONS,
            RemotePermissionType::ViewAuditLog => Permissions::VIEW_AUDIT_LOG,
            RemotePermissionType::PrioritySpeaker => Permissions::PRIORITY_SPEAKER,
            RemotePermissionType::Stream => Permissions::STREAM,
            RemotePermissionType::ViewChannel => Permissions::VIEW_CHANNEL,
            RemotePermissionType::SendMessages => Permissions::SEND_MESSAGES,
            RemotePermissionType::SendTtsMessages => Permissions::SEND_TTS_MESSAGES,
            RemotePermissionType::ManageMessages => Permissions::MANAGE_MESSAGES,
            RemotePermissionType::EmbedLinks => Permissions::EMBED_LINKS,
            RemotePermissionType::AttachFiles => Permissions::ATTACH_FILES,
            RemotePermissionType::ReadMessageHistory => Permissions::READ_MESSAGE_HISTORY,
            RemotePermissionType::MentionEveryone => Permissions::MENTION_EVERYONE,
            RemotePermissionType::UseExternalEmojis => Permissions::USE_EXTERNAL_EMOJIS,
            RemotePermissionType::ViewGuildInsights => Permissions::VIEW_GUILD_INSIGHTS,
            RemotePermissionType::Connect => Permissions::CONNECT,
            RemotePermissionType::Speak => Permissions::SPEAK,
            RemotePermissionType::MuteMembers => Permissions::MUTE_MEMBERS,
            RemotePermissionType::DeafenMembers => Permissions::DEAFEN_MEMBERS,
            RemotePermissionType::MoveMembers => Permissions::MOVE_MEMBERS,
            RemotePermissionType::UseVad => Permissions::USE_VAD,
            RemotePermissionType::ChangeNickname => Permissions::CHANGE_NICKNAME,
            RemotePermissionType::ManageNicknames => Permissions::MANAGE_NICKNAMES,
            RemotePermissionType::ManageRoles => Permissions::MANAGE_ROLES,
            RemotePermissionType::ManageWebhooks => Permissions::MANAGE_WEBHOOKS,
            RemotePermissionType::ManageGuildExpressions => Permissions::MANAGE_GUILD_EXPRESSIONS,
            RemotePermissionType::UseApplicationCommands => Permissions::USE_APPLICATION_COMMANDS,
            RemotePermissionType::RequestToSpeak => Permissions::REQUEST_TO_SPEAK,
            RemotePermissionType::ManageEvents => Permissions::MANAGE_EVENTS,
            RemotePermissionType::ManageThreads => Permissions::MANAGE_THREADS,
            RemotePermissionType::CreatePublicThreads => Permissions::CREATE_PUBLIC_THREADS,
            RemotePermissionType::CreatePrivateThreads => Permissions::CREATE_PRIVATE_THREADS,
            RemotePermissionType::UseExternalStickers => Permissions::USE_EXTERNAL_STICKERS,
            RemotePermissionType::SendMessagesInThreads => Permissions::SEND_MESSAGES_IN_THREADS,
            RemotePermissionType::UseEmbeddedActivities => Permissions::USE_EMBEDDED_ACTIVITIES,
            RemotePermissionType::ModerateMembers => Permissions::MODERATE_MEMBERS,
            RemotePermissionType::ViewCreatorMonetizationAnalytics => {
                Permissions::VIEW_CREATOR_MONETIZATION_ANALYTICS
            }
            RemotePermissionType::UseSoundboard => Permissions::USE_SOUNDBOARD,
            RemotePermissionType::CreateGuildExpressions => Permissions::CREATE_GUILD_EXPRESSIONS,
            RemotePermissionType::CreateEvents => Permissions::CREATE_EVENTS,
            RemotePermissionType::UseExternalSounds => Permissions::USE_EXTERNAL_SOUNDS,
            RemotePermissionType::SendVoiceMessages => Permissions::SEND_VOICE_MESSAGES,
            RemotePermissionType::SetVoiceChannelStatus => Permissions::SET_VOICE_CHANNEL_STATUS,
            RemotePermissionType::Unknown => Permissions::empty(),
        }
    }
}

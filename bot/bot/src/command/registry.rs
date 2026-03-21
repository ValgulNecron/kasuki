use anyhow::Result;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy)]
pub enum DiscordCommandType {
	ChatInput,
	SubCommand {
		parent: &'static str,
	},
	SubCommandGroup {
		parent: &'static str,
		group: &'static str,
	},
	User,
	Message,
	GuildChatInput {
		guild_id: u64,
	},
}

#[derive(Debug, Clone, Copy)]
pub enum ArgType {
	String,
	Integer,
	Boolean,
	User,
	Channel,
	Role,
	Mentionable,
	Number,
	Attachment,
}

#[derive(Debug, Clone, Copy)]
pub enum PermissionType {
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
}

impl From<PermissionType> for serenity::all::Permissions {
	fn from(p: PermissionType) -> Self {
		use serenity::all::Permissions;
		match p {
			PermissionType::CreateInstantInvite => Permissions::CREATE_INSTANT_INVITE,
			PermissionType::KickMembers => Permissions::KICK_MEMBERS,
			PermissionType::BanMembers => Permissions::BAN_MEMBERS,
			PermissionType::Administrator => Permissions::ADMINISTRATOR,
			PermissionType::ManageChannels => Permissions::MANAGE_CHANNELS,
			PermissionType::ManageGuild => Permissions::MANAGE_GUILD,
			PermissionType::AddReactions => Permissions::ADD_REACTIONS,
			PermissionType::ViewAuditLog => Permissions::VIEW_AUDIT_LOG,
			PermissionType::PrioritySpeaker => Permissions::PRIORITY_SPEAKER,
			PermissionType::Stream => Permissions::STREAM,
			PermissionType::ViewChannel => Permissions::VIEW_CHANNEL,
			PermissionType::SendMessages => Permissions::SEND_MESSAGES,
			PermissionType::SendTtsMessages => Permissions::SEND_TTS_MESSAGES,
			PermissionType::ManageMessages => Permissions::MANAGE_MESSAGES,
			PermissionType::EmbedLinks => Permissions::EMBED_LINKS,
			PermissionType::AttachFiles => Permissions::ATTACH_FILES,
			PermissionType::ReadMessageHistory => Permissions::READ_MESSAGE_HISTORY,
			PermissionType::MentionEveryone => Permissions::MENTION_EVERYONE,
			PermissionType::UseExternalEmojis => Permissions::USE_EXTERNAL_EMOJIS,
			PermissionType::ViewGuildInsights => Permissions::VIEW_GUILD_INSIGHTS,
			PermissionType::Connect => Permissions::CONNECT,
			PermissionType::Speak => Permissions::SPEAK,
			PermissionType::MuteMembers => Permissions::MUTE_MEMBERS,
			PermissionType::DeafenMembers => Permissions::DEAFEN_MEMBERS,
			PermissionType::MoveMembers => Permissions::MOVE_MEMBERS,
			PermissionType::UseVad => Permissions::USE_VAD,
			PermissionType::ChangeNickname => Permissions::CHANGE_NICKNAME,
			PermissionType::ManageNicknames => Permissions::MANAGE_NICKNAMES,
			PermissionType::ManageRoles => Permissions::MANAGE_ROLES,
			PermissionType::ManageWebhooks => Permissions::MANAGE_WEBHOOKS,
			PermissionType::ManageGuildExpressions => Permissions::MANAGE_GUILD_EXPRESSIONS,
			PermissionType::UseApplicationCommands => Permissions::USE_APPLICATION_COMMANDS,
			PermissionType::RequestToSpeak => Permissions::REQUEST_TO_SPEAK,
			PermissionType::ManageEvents => Permissions::MANAGE_EVENTS,
			PermissionType::ManageThreads => Permissions::MANAGE_THREADS,
			PermissionType::CreatePublicThreads => Permissions::CREATE_PUBLIC_THREADS,
			PermissionType::CreatePrivateThreads => Permissions::CREATE_PRIVATE_THREADS,
			PermissionType::UseExternalStickers => Permissions::USE_EXTERNAL_STICKERS,
			PermissionType::SendMessagesInThreads => Permissions::SEND_MESSAGES_IN_THREADS,
			PermissionType::UseEmbeddedActivities => Permissions::USE_EMBEDDED_ACTIVITIES,
			PermissionType::ModerateMembers => Permissions::MODERATE_MEMBERS,
			PermissionType::ViewCreatorMonetizationAnalytics => {
				Permissions::VIEW_CREATOR_MONETIZATION_ANALYTICS
			},
			PermissionType::UseSoundboard => Permissions::USE_SOUNDBOARD,
			PermissionType::CreateGuildExpressions => Permissions::CREATE_GUILD_EXPRESSIONS,
			PermissionType::CreateEvents => Permissions::CREATE_EVENTS,
			PermissionType::UseExternalSounds => Permissions::USE_EXTERNAL_SOUNDS,
			PermissionType::SendVoiceMessages => Permissions::SEND_VOICE_MESSAGES,
			PermissionType::SetVoiceChannelStatus => Permissions::SET_VOICE_CHANNEL_STATUS,
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub enum ContextType {
	Guild,
	BotDm,
	PrivateChannel,
}

impl From<ContextType> for serenity::all::InteractionContext {
	fn from(c: ContextType) -> Self {
		match c {
			ContextType::Guild => serenity::all::InteractionContext::Guild,
			ContextType::BotDm => serenity::all::InteractionContext::BotDm,
			ContextType::PrivateChannel => serenity::all::InteractionContext::PrivateChannel,
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub enum InstallType {
	Guild,
	User,
}

impl From<InstallType> for serenity::all::InstallationContext {
	fn from(i: InstallType) -> Self {
		match i {
			InstallType::Guild => serenity::all::InstallationContext::Guild,
			InstallType::User => serenity::all::InstallationContext::User,
		}
	}
}

impl From<ArgType> for serenity::all::CommandOptionType {
	fn from(a: ArgType) -> Self {
		match a {
			ArgType::String => serenity::all::CommandOptionType::String,
			ArgType::Integer => serenity::all::CommandOptionType::Integer,
			ArgType::Boolean => serenity::all::CommandOptionType::Boolean,
			ArgType::User => serenity::all::CommandOptionType::User,
			ArgType::Channel => serenity::all::CommandOptionType::Channel,
			ArgType::Role => serenity::all::CommandOptionType::Role,
			ArgType::Mentionable => serenity::all::CommandOptionType::Mentionable,
			ArgType::Number => serenity::all::CommandOptionType::Number,
			ArgType::Attachment => serenity::all::CommandOptionType::Attachment,
		}
	}
}

#[derive(Debug)]
pub struct ArgDef {
	pub name: &'static str,
	pub desc: &'static str,
	pub arg_type: ArgType,
	pub required: bool,
	pub autocomplete: bool,
	pub choices: &'static [ChoiceDef],
}

#[derive(Debug)]
pub struct ChoiceDef {
	pub name: &'static str,
}

#[derive(Debug)]
pub struct CommandMeta {
	pub name: &'static str,
	pub desc: &'static str,
	pub command_type: DiscordCommandType,
	pub nsfw: bool,
	pub permissions: &'static [PermissionType],
	pub contexts: &'static [ContextType],
	pub install_contexts: &'static [InstallType],
	pub args: &'static [ArgDef],
}

pub trait SlashCommand: Send + Sync + 'static {
	fn meta(&self) -> &'static CommandMeta;
	fn dispatch_key(&self) -> &'static str;
	fn run<'a>(
		&'a self, ctx: &'a SerenityContext, interaction: &'a CommandInteraction,
		full_command_name: &'a str,
	) -> Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>>;
}

// inventory collects all #[slash_command]-annotated commands across the crate at link time
inventory::collect!(&'static dyn SlashCommand);

pub struct ParentCommand {
	pub name: &'static str,
	pub desc: &'static str,
	pub nsfw: bool,
	pub permissions: &'static [PermissionType],
	pub contexts: &'static [ContextType],
	pub install_contexts: &'static [InstallType],
	pub groups: &'static [GroupDef],
}

pub struct GroupDef {
	pub name: &'static str,
	pub desc: &'static str,
}

// Parent commands define the top-level grouping for subcommand hierarchies
inventory::collect!(&'static ParentCommand);

// OnceLock guarantees thread-safe one-time init without async — unlike Mutex/RwLock, reads
// after init are zero-cost (no locking). Separate maps per command type for O(1) dispatch.
static SLASH_REGISTRY: OnceLock<HashMap<String, &'static dyn SlashCommand>> = OnceLock::new();
static USER_REGISTRY: OnceLock<HashMap<String, &'static dyn SlashCommand>> = OnceLock::new();
static MESSAGE_REGISTRY: OnceLock<HashMap<String, &'static dyn SlashCommand>> = OnceLock::new();
// Guild commands are stored as a Vec (not a map) because they need to be iterated for per-guild registration
static GUILD_REGISTRY: OnceLock<Vec<&'static dyn SlashCommand>> = OnceLock::new();

fn build_registries() {
	let mut slash = HashMap::new();
	let mut user = HashMap::new();
	let mut message = HashMap::new();
	let mut guild = Vec::new();

	for cmd in inventory::iter::<&'static dyn SlashCommand> {
		let meta = cmd.meta();
		let key = cmd.dispatch_key().to_string();
		match meta.command_type {
			DiscordCommandType::ChatInput
			| DiscordCommandType::SubCommand { .. }
			| DiscordCommandType::SubCommandGroup { .. } => {
				slash.insert(key, *cmd);
			},
			DiscordCommandType::User => {
				user.insert(key, *cmd);
			},
			DiscordCommandType::Message => {
				message.insert(key, *cmd);
			},
			DiscordCommandType::GuildChatInput { .. } => {
				// Guild commands go into both maps: slash for dispatch, guild vec for registration
				slash.insert(key, *cmd);
				guild.push(*cmd);
			},
		}
	}

	// Ignoring set() results — if already initialized (race), the existing values win
	let _ = SLASH_REGISTRY.set(slash);
	let _ = USER_REGISTRY.set(user);
	let _ = MESSAGE_REGISTRY.set(message);
	let _ = GUILD_REGISTRY.set(guild);
}

pub fn get_slash_registry() -> &'static HashMap<String, &'static dyn SlashCommand> {
	// get_or_init closure is the fallback if init_registries() wasn't called at startup.
	// build_registries() populates the real OnceLock; the closure return is only used if
	// set() failed (shouldn't happen), so unwrap_or_default is a safe fallback.
	SLASH_REGISTRY.get_or_init(|| {
		build_registries();
		SLASH_REGISTRY.get().cloned().unwrap_or_default()
	})
}

pub fn get_user_registry() -> &'static HashMap<String, &'static dyn SlashCommand> {
	USER_REGISTRY.get_or_init(|| {
		build_registries();
		USER_REGISTRY.get().cloned().unwrap_or_default()
	})
}

pub fn get_message_registry() -> &'static HashMap<String, &'static dyn SlashCommand> {
	MESSAGE_REGISTRY.get_or_init(|| {
		build_registries();
		MESSAGE_REGISTRY.get().cloned().unwrap_or_default()
	})
}

pub fn get_guild_commands() -> &'static Vec<&'static dyn SlashCommand> {
	GUILD_REGISTRY.get_or_init(|| {
		build_registries();
		GUILD_REGISTRY.get().cloned().unwrap_or_default()
	})
}

/// Initialize all registries. Call once at startup.
pub fn init_registries() {
	build_registries();
}

/// Get all slash command entries (for registration purposes)
pub fn all_slash_commands() -> impl Iterator<Item = &'static &'static dyn SlashCommand> {
	inventory::iter::<&'static dyn SlashCommand>.into_iter()
}

/// Get all parent commands (for registration purposes)
pub fn all_parent_commands() -> impl Iterator<Item = &'static &'static ParentCommand> {
	inventory::iter::<&'static ParentCommand>.into_iter()
}

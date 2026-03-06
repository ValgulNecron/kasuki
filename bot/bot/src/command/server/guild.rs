//! Documentation for the GuildCommand struct and its implementation.
//!
//! # Overview
//! The `GuildCommand` struct and its `Command` trait implementation are designed
//! to handle Discord bot commands within a guild (server) context. This implementation
//! retrieves detailed information about a guild and constructs response embeds
//! containing guild-specific attributes and statistics.
//!
//! ## Struct: GuildCommand
//! The `GuildCommand` struct holds the context and interaction information necessary
//! to process the command.
//!
//! ### Fields
//! - `ctx` (SerenityContext): Provides access to the bot's context, including
//!    server data, HTTP client access, and more.
//! - `command_interaction` (CommandInteraction): Represents the information about the
//!    received command interaction.
//!
//! ## Trait: Command
//!
//! The `GuildCommand` struct implements the `Command` trait, which defines the
//! structure and behavior for executing guild-related commands.
//!
//! ### Methods
//!
//! #### `get_ctx(&self) -> &SerenityContext`
//! Returns the bot's context (`serenity::Context`) that contains essential
//! runtime and state details required for command execution.
//!
//! #### `get_command_interaction(&self) -> &CommandInteraction`
//! Returns the command interaction (`serenity::CommandInteraction`) that
//! contains the command invocation details from Discord.
//!
//! #### `get_contents(&self) -> Result<Vec<EmbedContent<'_, '_>>>`
//! This asynchronous method prepares and retrieves a collection of embed content
//! detailing information about the guild where the command was invoked.
//!
//! ##### Behavior:
//! 1. **Fetch Guild Details**:
//!    - Retrieves the guild's ID and localized information.
//!    - Attempts to fetch additional guild metadata, such as member counts,
//!      creation date, channels, roles, and premium features.
//! 2. **Embed Construction**:
//!    - Constructs an embed with fields for guild-specific details such as:
//!        - Guild name
//!        - Member stats
//!        - Online members
//!        - Creation date
//!        - Preferred locale
//!        - Premium tier and subscription count
//!        - Roles and channels count
//!        - NSFW level
//!        - Verification level
//!    - Optional elements like the guild's avatar and banner are also
//!      included if available.
//! 3. **Return Value**:
//!    - Returns a vector containing a single `EmbedContent` object filled with
//!      the guild's data.
//!
//! ##### Returns:
//! - `Ok(Vec<EmbedContent<'_, '_>>)` containing the embed with guild information.
//! - `Err(anyhow::Error)` if any error occurs during command execution (e.g.,
//! failure to fetch guild details).
//!
//! ## Error Handling
//! If required information, such as guild ID or metadata, cannot be fetched,
//! the method returns an error (`anyhow::Error`). This allows for graceful failure
//! with meaningful error messages.
//!
//! ## Example Use Case
//! When a user invokes a command to display guild details, the bot uses this
//! implementation to gather and format relevant guild information in an embed.
//!
//! ## Dependencies:
//! - `serenity`: A Rust library for interacting with Discord's API.
//! - `anyhow`: For error handling.
//! - `EmbedContent`: A custom type used to construct rich embed responses.
//!
//! ## Note:
//! This implementation assumes that the bot is running with appropriate scopes
//! and permissions to access guild data such as its metadata, channels, and roles.
use anyhow::anyhow;

use crate::command::command::CommandRun;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use serenity::nonmax::NonMaxU64;
use shared::localization::{get_language_identifier, Loader, USABLE_LOCALES};

#[slash_command(
	name = "guild", desc = "Get info of the guild.",
	command_type = SubCommand(parent = "server"),
	contexts = [Guild, PrivateChannel],
	install_contexts = [Guild],
)]
async fn guild_command(self_: GuildCommand) -> Result<EmbedsContents<'_>> {
	let ctx = self_.get_ctx();
	let bot_data = ctx.data::<BotData>().clone();
	let command_interaction = self_.get_command_interaction();

	// Retrieve the guild ID from the command interaction
	let guild_id_str = match command_interaction.guild_id {
		Some(id) => id.to_string(),
		None => String::from("0"),
	};
	let db_connection = bot_data.db_connection.clone();

	// Load the localized guild information
	let lang_id = get_language_identifier(guild_id_str, db_connection).await;

	// Retrieve the guild ID from the command interaction or return an error if it does not exist
	let guild_id = command_interaction.guild_id.ok_or(anyhow!("No guild ID"))?;

	// Retrieve the guild's information or return an error if it could not be retrieved
	let guild = guild_id.to_partial_guild_with_counts(&ctx.http).await?;

	// Retrieve various details about the guild
	let channels = guild.id.channels(&ctx.http).await.unwrap_or_default().len();

	let guild_id = guild.id;

	let guild_name = guild.name.clone();

	let max_member = guild.max_members.unwrap_or_default();

	let actual_member = guild.approximate_member_count.unwrap_or_default();

	let online_member = guild.approximate_presence_count.unwrap_or_default();

	let max_online = guild
		.max_presences
		.unwrap_or(NonMaxU64::new(25000).unwrap_or_default());

	let guild_banner = guild.banner_url();

	let guild_avatar = guild.icon_url();

	let guild_lang = guild.preferred_locale;

	let guild_premium = guild.premium_tier;

	let guild_sub = guild.premium_subscription_count.unwrap_or_default();

	let guild_nsfw = guild.nsfw_level;

	let creation_date = format!("<t:{}:F>", guild.id.created_at().unix_timestamp());

	let owner = guild
		.owner_id
		.to_user(&ctx.http)
		.await
		.map(|u| u.tag().to_string())
		.unwrap_or_default();

	let roles = guild.roles.len();

	let verification_level = guild.verification_level;

	// Initialize a vector to store the fields for the embed
	let mut fields: Vec<(String, String, bool)> = Vec::new();

	// Add the fields to the vector
	fields.push((
		USABLE_LOCALES.lookup(&lang_id, "server_guild-guild_id"),
		guild_id.to_string(),
		true,
	));

	fields.push((
		USABLE_LOCALES.lookup(&lang_id, "server_guild-guild_name"),
		guild_name.to_string(),
		true,
	));

	fields.push((
		USABLE_LOCALES.lookup(&lang_id, "server_guild-member"),
		format!("{}/{}", actual_member, max_member),
		true,
	));

	fields.push((
		USABLE_LOCALES.lookup(&lang_id, "server_guild-online"),
		format!("{}/{}", online_member, max_online),
		true,
	));

	fields.push((
		USABLE_LOCALES.lookup(&lang_id, "server_guild-creation_date"),
		creation_date,
		true,
	));

	fields.push((
		USABLE_LOCALES.lookup(&lang_id, "server_guild-lang"),
		guild_lang.to_string(),
		true,
	));

	fields.push((
		USABLE_LOCALES.lookup(&lang_id, "server_guild-premium"),
		format!("{:?}", guild_premium),
		true,
	));

	fields.push((
		USABLE_LOCALES.lookup(&lang_id, "server_guild-sub"),
		guild_sub.to_string(),
		true,
	));

	fields.push((
		USABLE_LOCALES.lookup(&lang_id, "server_guild-nsfw"),
		format!("{:?}", guild_nsfw),
		true,
	));

	fields.push((
		USABLE_LOCALES.lookup(&lang_id, "server_guild-owner"),
		owner.to_string(),
		true,
	));

	fields.push((
		USABLE_LOCALES.lookup(&lang_id, "server_guild-roles"),
		roles.to_string(),
		true,
	));

	fields.push((
		USABLE_LOCALES.lookup(&lang_id, "server_guild-channels"),
		channels.to_string(),
		true,
	));

	fields.push((
		USABLE_LOCALES.lookup(&lang_id, "server_guild-verification_level"),
		format!("{:?}", verification_level),
		true,
	));

	// Construct the embed for the response

	let mut embed_content = EmbedContent::new(String::new()).fields(fields);

	// Add the guild's avatar to the embed if it exists
	if guild_avatar.is_some() {
		embed_content = embed_content.thumbnail(guild_avatar.unwrap())
	}

	// Add the guild's banner to the embed if it exists
	if guild_banner.is_some() {
		embed_content = embed_content.images_url(guild_banner.unwrap())
	}

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}

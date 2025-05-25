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
use anyhow::{Result, anyhow};

use crate::command::command::Command;
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use crate::structure::message::server::guild::load_localization_guild;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use serenity::nonmax::NonMaxU64;

/// A structure representing a command within a guild context on Discord.
///
/// This structure is used to encapsulate the context and interaction data
/// associated with a command executed in a Discord guild.
///
/// # Fields
///
/// * `ctx` - Represents the context of the command, providing access to the
///           bot's state and utility functions. This is an instance of
///           `SerenityContext`, which enables interaction with the Discord API.
///
/// * `command_interaction` - Represents the interaction data associated with
///                           the executed command. This holds information
///                           about the command, the user who invoked it,
///                           and other relevant interaction details.
pub struct GuildCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for GuildCommand {
	/// Retrieves a reference to the `SerenityContext` associated with the current object.
	///
	/// # Returns
	/// A reference to the `SerenityContext` stored in the current object.
	///
	/// # Example
	/// ```rust
	/// let context = object.get_ctx();
	/// // Use the retrieved `SerenityContext`
	/// ```
	///
	/// This function can be used to access the Discord bot context for performing various operations
	/// within the bot's lifecycle.
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` instance associated with this object.
	///
	/// This method provides access to the `command_interaction` field of the implementing structure,
	/// allowing you to inspect or manipulate information related to a command interaction.
	///
	/// # Returns
	/// - A reference to the `CommandInteraction` object.
	///
	/// # Example
	/// ```
	/// let interaction = obj.get_command_interaction();
	/// // Use the `interaction` for further operations
	/// ```
	///
	/// # Panics
	/// This method does not panic.
	///
	/// # Safety
	/// This method is safe to call as it only returns a reference to an internal field.
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Asynchronously retrieves embed content representing information about a guild.
	///
	/// This method performs the following tasks:
	/// 1. Retrieves the context and necessary bot data.
	/// 2. Extracts the guild ID from the command interaction, returning "0" if not present.
	/// 3. Loads the localized guild information using the guild ID.
	/// 4. Retrieves detailed information about the guild, including its:
	///    - Name
	///    - Member counts (actual, maximum)
	///    - Online member count and max online capacity
	///    - Creation date
	///    - Owner information
	///    - Roles count
	///    - Channels count
	///    - Verification level
	///    - NSFW level
	///    - Banner image
	///    - Avatar image
	/// 5. Constructs an embed with the guild's information and fields, and optionally
	///    includes the guild banner and avatar if available.
	///
	/// # Returns
	/// A vector containing a single `EmbedContent` instance represented as a custom response.
	///
	/// # Errors
	/// This method returns an error if:
	/// - The guild ID is unavailable or invalid.
	/// - The guild's information could not be retrieved from the context or API.
	/// - The localization or database operation fails.
	///
	/// # Example
	/// ```rust
	/// let embed_data = guild_data.get_contents().await;
	/// if let Ok(embeds) = embed_data {
	///     // Perform further operations with embed data
	/// }
	/// ```
	async fn get_contents(&self) -> Result<EmbedsContents> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();
		let config = bot_data.config.clone();

		// Retrieve the guild ID from the command interaction
		let guild_id = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};

		// Load the localized guild information
		let guild_localised = load_localization_guild(guild_id, config.db.clone()).await?;

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
			.map(|u| u.tag())
			.unwrap_or_default();

		let roles = guild.roles.len();

		let verification_level = guild.verification_level;

		// Initialize a vector to store the fields for the embed
		let mut fields: Vec<(String, String, bool)> = Vec::new();

		// Add the fields to the vector
		fields.push((guild_localised.guild_id, guild_id.to_string(), true));

		fields.push((guild_localised.guild_name, guild_name.to_string(), true));

		fields.push((
			guild_localised.member,
			format!("{}/{}", actual_member, max_member),
			true,
		));

		fields.push((
			guild_localised.online,
			format!("{}/{}", online_member, max_online),
			true,
		));

		fields.push((guild_localised.creation_date, creation_date, true));

		fields.push((guild_localised.lang, guild_lang.to_string(), true));

		fields.push((
			guild_localised.premium,
			format!("{:?}", guild_premium),
			true,
		));

		fields.push((guild_localised.sub, guild_sub.to_string(), true));

		fields.push((guild_localised.nsfw, format!("{:?}", guild_nsfw), true));

		fields.push((guild_localised.owner, owner, true));

		fields.push((guild_localised.roles, roles.to_string(), true));

		fields.push((guild_localised.channels, channels.to_string(), true));

		fields.push((
			guild_localised.verification_level,
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

		let embed_contents = EmbedsContents::new(CommandType::First, vec![embed_content]);

		Ok(embed_contents)
	}
}

//! The `RegisterCommand` struct is responsible for handling the "register" command interaction
//! within a Discord bot. It is part of a bot infrastructure that integrates with the AniList API
//! and a database for user registrations.
//!
//! This command allows users to register their AniList account with the bot, storing a mapping
//! between their Discord ID and AniList ID in the database.
//!
//! # Fields
//! - `ctx`: The `SerenityContext` that represents the current bot state and provides access to shared
//!          data like the database connection, configuration, etc.
//! - `command_interaction`: The command interaction event received from Discord, containing details
//!                          about the invoked command (i.e., the user, arguments, and guild information).
//!
//! This struct implements the `Command` trait, defining the behavior and response of the "register" command.
use anyhow::anyhow;

use sea_orm::ActiveValue::Set;
use sea_orm::EntityTrait;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use small_fixed_array::FixedString;

use crate::command::anilist_user::user::get_user;
use crate::command::command::{Command, CommandRun};
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::database::prelude::RegisteredUser;
use crate::database::registered_user::{ActiveModel, Column};
use crate::event_handler::BotData;
use crate::helper::get_option::command::get_option_map_string;
use crate::structure::message::anilist_user::register::load_localization_register;
use crate::structure::run::anilist::user::{User, get_color, get_user_url};

/// A structure representing a command registration action within the application.
///
/// This struct encapsulates the necessary context and interaction details required
/// to handle and process a slash command registration event in a Discord bot.
///
/// # Fields
///
/// * `ctx` - A `SerenityContext` that provides access to details about the bot's
///   runtime environment, such as the shard manager, cache, HTTP client, and other
///   utilities required to interact with Discord's API.
///
/// * `command_interaction` - A `CommandInteraction` instance that contains detailed
///   information about the specific slash command interaction, including the user's
///   input, command arguments, and the originating context of the interaction.
///
/// # Usage
///
/// This struct is typically used to handle slash commands in Discord bots built
/// with the Serenity library. When a user invokes a slash command, the bot receives
/// an interaction event, which can then be represented by this struct to manage
/// the interaction.
///
/// # Example
pub struct RegisterCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for RegisterCommand {
	/// Retrieves a reference to the `SerenityContext`.
	///
	/// This function provides access to the current `SerenityContext` instance associated
	/// with the object. The `SerenityContext` typically contains important data and utilities
	/// required for interacting with the Discord API.
	///
	/// # Returns
	/// A reference to the `SerenityContext` instance.
	///
	/// # Example
	/// ```rust
	/// let ctx = my_object.get_ctx();
	/// // Use the context to interact with Discord API or perform operations.
	/// ```
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` instance associated with the current object.
	///
	/// # Returns
	///
	/// A reference to the `CommandInteraction` instance (`&CommandInteraction`) stored within the object.
	///
	/// # Example
	///
	/// ```rust
	/// let interaction = my_object.get_command_interaction();
	/// // Use `interaction` as needed
	/// ```
	///
	/// This method is useful when you need to access or inspect the `CommandInteraction` data
	/// without taking ownership of it.
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Asynchronously fetches and composes user-related embed content for a Discord bot.
	///
	/// This function retrieves user information from the AniList service using the bot's context.
	/// The retrieved data is used to create an embed with user-specific details for displaying
	/// in Discord. It also handles updating or inserting user data into the bot's database.
	///
	/// # Returns
	/// * `Result<Vec<EmbedContent<'_, '_>>>` - A vector of embed content that can be sent as part
	///   of a message, or an error if the operation fails.
	///
	/// # Errors
	/// This function will return an error in the following circumstances:
	/// - If no username is provided in the command interaction.
	/// - If fetching user data from AniList fails.
	/// - If localization data cannot be loaded.
	/// - If database operations (inserting/registering the user) fail.
	///
	/// # Workflow
	/// 1. The function retrieves context and bot-specific resources (e.g., cache, database connection).
	///
	async fn get_contents<'a>(&'a self) -> anyhow::Result<EmbedsContents<'a>> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();

		let anilist_cache = bot_data.anilist_cache.clone();
		let connection = bot_data.db_connection.clone();
		let config = bot_data.config.clone();

		let map = get_option_map_string(command_interaction);

		self.defer().await?;

		let value = map
			.get(&FixedString::from_str_trunc("username"))
			.ok_or(anyhow!("No username provided"))?;

		// Fetch the user data from AniList
		let user_data: User = get_user(value, anilist_cache).await?;

		// Retrieve the guild ID from the command interaction
		let guild_id = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};

		// Load the localized register strings
		let register_localised = load_localization_register(guild_id, config.db.clone()).await?;

		// Retrieve the user's Discord ID and username
		let user_id = &command_interaction.user.id.to_string();

		let username = &command_interaction.user.name;

		RegisteredUser::insert(ActiveModel {
			user_id: Set(user_id.to_string()),
			anilist_id: Set(user_data.id),
			..Default::default()
		})
		.on_conflict(
			sea_orm::sea_query::OnConflict::column(Column::AnilistId)
				.update_column(Column::AnilistId)
				.to_owned(),
		)
		.exec(&*connection)
		.await?;

		// Construct the description for the embed
		let desc = register_localised
			.desc
			.replace("$user$", username.as_str())
			.replace("$id$", user_id)
			.replace("$anilist$", user_data.name.clone().as_str());

		let embed_content = EmbedContent::new(user_data.clone().name)
			.description(desc)
			.thumbnail(user_data.clone().avatar.unwrap().large.unwrap())
			.url(get_user_url(&user_data.id))
			.colour(get_color(user_data.clone()));

		let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

		Ok(embed_contents)
	}
}

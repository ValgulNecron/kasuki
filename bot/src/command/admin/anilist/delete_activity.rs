//! This module implements functionality to manage the deletion of activities related to anime
//! within a Discord bot using the Serenity and SeaORM libraries. It defines the `DeleteActivityCommand`,
//! enabling the ability to delete activities associated with an anime, and the helper function `remove_activity` to handle database operations.
use crate::command::admin::anilist::add_activity::{get_minimal_anime_media, get_name};
use crate::command::command::{Command, CommandRun};
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::config::DbConfig;
use crate::database::prelude::ActivityData;
use crate::event_handler::BotData;
use crate::get_url;
use crate::helper::get_option::subcommand_group::get_option_map_string_subcommand_group;
use crate::structure::message::admin::anilist::delete_activity::load_localization_delete_activity;
use anyhow::{Result, anyhow};
use sea_orm::ColumnTrait;
use sea_orm::{EntityTrait, ModelTrait, QueryFilter};
use serenity::all::{CommandInteraction, Context as SerenityContext};

/// A struct representing the command to delete an activity in a Discord bot.
///
/// This struct contains the necessary context and interaction information required
/// to process a delete activity command issued by a user.
///
/// # Fields
/// * `ctx` - The context of the bot provided by the Serenity framework.
///           This includes data such as the client cache, HTTP instance, and event handlers.
/// * `command_interaction` - The interaction data representing the slash command
///                            or user input used to trigger the delete activity command.
///
/// # Example
/// ```rust
/// use serenity::prelude::*;
/// use serenity::model::application::interaction::application_command::CommandInteraction;
///
/// let delete_command = DeleteActivityCommand {
///     ctx: SerenityContext::new(),
///     command_interaction: CommandInteraction::default(),
/// };
/// // Perform deletion logic using the command's context and interaction.
/// ```
pub struct DeleteActivityCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for DeleteActivityCommand {
	/// Retrieves a reference to the `SerenityContext` stored within the current instance.
	///
	/// # Returns
	/// A reference to the `SerenityContext` associated with this instance.
	///
	/// # Usage
	/// This function is useful for accessing the context needed to interact with
	/// the Discord API or perform various bot operations.
	///
	/// # Example
	/// ```rust
	/// let context = instance.get_ctx();
	/// // Use context to perform bot operations
	/// ```
	///
	/// # Note
	/// Ensure the instance is correctly initialized with a `SerenityContext`
	/// before calling this method.
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` instance contained within the struct.
	///
	/// # Returns
	/// A shared reference to the `CommandInteraction` instance.
	///
	/// # Example
	/// ```rust
	/// let interaction = my_struct.get_command_interaction();
	/// // Use `interaction` as needed
	/// ```
	///
	/// # Notes
	/// - This method provides read-only access to `command_interaction`.
	/// - Ensure that the returned reference is used within the lifetime of `self`.
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Asynchronously retrieves embed content data based on a given anime name and performs a delete activity operation.
	///
	/// This function executes the following steps:
	/// 1. Retrieves the bot and cached data needed for processing.
	/// 2. Parses the input command to extract the anime name provided by the user.
	/// 3. Identifies the guild ID from the interaction, defaulting to "1" if none is provided.
	/// 4. Loads localized text for the activity delete operation based on the guild's settings.
	/// 5. Queries AniList for minimal anime media information based on the anime name.
	/// 6. Defers the bot's response to indicate the operation is ongoing.
	/// 7. Deletes the associated activity record from the database if the anime is successfully located.
	/// 8. Constructs the embed content, including a success message and relevant links.
	///
	/// # Returns
	/// Returns a `Result` with a vector of `EmbedContent` if successful. Otherwise, returns an error.
	///
	/// # Errors
	/// Returns an error for the following scenarios:
	/// - If asynchronous operations such as fetching data from the database or AniList fail.
	/// - If the anime media information cannot be retrieved or is incomplete.
	/// - If localization data for the delete activity operation cannot be loaded.
	///
	/// # Examples
	/// ```ignore
	/// let embed_contents = instance.get_contents().await?;
	/// for content in embed_contents {
	///     println!("{:?}", content);
	/// }
	/// ```
	///
	/// # Dependencies
	/// The function relies on external services and modules for:
	/// - Fetching anime media information from AniList.
	/// - Database operations for managing activity records.
	/// - Localization functions for internationalization and localized user feedback.
	async fn get_contents<'a>(&'a self) -> anyhow::Result<EmbedsContents<'a>> {
		let command_interaction = self.get_command_interaction();
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let config = bot_data.config.clone();
		let anilist_cache = bot_data.anilist_cache.clone();

		let map = get_option_map_string_subcommand_group(&command_interaction);
		let anime = map
			.get(&String::from("anime_name"))
			.cloned()
			.unwrap_or(String::new());

		let guild_id = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("1"),
		};

		let delete_activity_localised_text =
			load_localization_delete_activity(guild_id.clone(), config.db.clone());
		let media = get_minimal_anime_media(anime.to_string(), anilist_cache);

		self.defer().await?;

		let media = media.await?;
		let anime_id = media.id;

		remove_activity(guild_id.as_str(), &anime_id, config.db.clone()).await?;

		let title = media
			.title
			.ok_or(anyhow!(format!("Anime with id {} not found", anime_id)))?;
		let anime_name = get_name(title);

		let url = format!("https://anilist.co/anime/{}", anime_id);

		let delete_activity_localised_text = delete_activity_localised_text.await?;
		let embed_content = EmbedContent::new(delete_activity_localised_text.success)
			.description(
				delete_activity_localised_text
					.success_desc
					.replace("$anime$", anime_name.as_str()),
			)
			.url(url);

		let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

		Ok(embed_contents)
	}
}

/// Asynchronously removes an activity entry from the database based on the provided guild ID and anime ID.
///
/// # Arguments
/// - `guild_id`: A reference to a string slice representing the ID of the Discord guild/server.
/// - `anime_id`: A reference to an integer representing the ID of the anime to be removed.
/// - `db_config`: A `DbConfig` instance containing the connection configuration for the database.
///
/// # Returns
/// - `Result<()>`: Returns an `Ok(())` upon successful removal of the activity. Otherwise, returns an `Err` if any error occurs during the operation.
///
/// # Errors
/// This function may return an error if:
/// - The database connection fails.
/// - The query to find the activity data fails.
/// - The anime with the specified ID does not exist in the database (returns an error with a message).
/// - The deletion of the activity entry fails.
///
/// # Examples
/// ```rust
/// let guild_id = "123456789012345678";
/// let anime_id = 42;
/// let db_config = DbConfig::new("database_url");
///
/// match remove_activity(guild_id, &anime_id, db_config).await {
///     Ok(_) => println!("Activity successfully removed."),
///     Err(err) => eprintln!("Failed to remove activity: {}", err),
/// }
/// ```
async fn remove_activity(guild_id: &str, anime_id: &i32, db_config: DbConfig) -> Result<()> {
	let connection = sea_orm::Database::connect(get_url(db_config.clone())).await?;

	let activity = ActivityData::find()
		.filter(crate::database::activity_data::Column::ServerId.eq(guild_id))
		.filter(crate::database::activity_data::Column::AnimeId.eq(anime_id.to_string()))
		.one(&connection)
		.await?
		.ok_or(anyhow!(format!("Anime with id {} not found", anime_id)))?;

	activity.delete(&connection).await?;

	Ok(())
}

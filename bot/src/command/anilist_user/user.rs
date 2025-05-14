//! The `UserCommand` struct is used to handle the execution of a user-related command.
//! This command fetches user information from AniList and optionally retrieves user-specific
//! data stored in the database.
//!
//! # Fields
//! - `ctx`: The Serenity context required to interact with the bot.
//! - `command_interaction`: Details about the command interaction being processed.
use crate::command::command::{Command, CommandRun, EmbedContent};
use crate::database::prelude::RegisteredUser;
use crate::database::registered_user::Column;
use crate::event_handler::BotData;
use crate::get_url;
use crate::helper::get_option::command::get_option_map_string;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::run::anilist::user;
use crate::structure::run::anilist::user::{
	User, UserQueryId, UserQueryIdVariables, UserQuerySearch, UserQuerySearchVariables,
};
use anyhow::{Result, anyhow};
use cynic::{GraphQlResponse, QueryBuilder};
use moka::future::Cache;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use small_fixed_array::FixedString;
use std::sync::Arc;
use tokio::sync::RwLock;

/// A struct representing a user command in a Discord bot.
///
/// The `UserCommand` struct encapsulates the context and interaction
/// details for a user command executed in a Discord server. It provides
/// access to the bot's runtime context and the specific interaction data
/// for handling the command.
///
/// # Fields
///
/// * `ctx` - A `SerenityContext` instance which provides access to the bot's
///           runtime context, including the HTTP client, cache, and shard manager.
///           This enables interaction with Discord and the ability to access
///           additional resources and configuration.
///
/// * `command_interaction` - A `CommandInteraction` instance representing
///                           the interaction triggered by the user command.
///                           This contains data specific to the command, such as
///                           user input, the command name, and other metadata.
///
/// # Example
///
/// ```rust
/// use my_bot::commands::UserCommand;
///
/// // Creating a `UserCommand` instance
/// fn handle_user_command(user_command: UserCommand) {
///     let context = user_command.ctx;
///     let interaction = user_command.command_interaction;
///     
///     // Use context and interaction to process the command...
/// }
/// ```
///
/// This struct facilitates the processing of user commands in a structured and
/// easily accessible way, leveraging the Serenity framework.
pub struct UserCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for UserCommand {
	/// Returns a reference to the `SerenityContext` associated with the current instance.
	///
	/// # Returns
	///
	/// A reference to the `SerenityContext` (`&SerenityContext`) that is stored within the current instance.
	///
	/// # Examples
	///
	/// ```rust
	/// let ctx = instance.get_ctx();
	/// // Use `ctx` to perform actions with the Serenity context.
	/// ```
	///
	/// This method is useful for accessing the Serenity Discord context to interact with
	/// the Discord API, such as sending messages, managing guilds, or handling events.
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` associated with the current instance.
	///
	/// # Returns
	///
	/// A reference to the `CommandInteraction` object.
	///
	/// # Example
	/// ```
	/// let interaction = instance.get_command_interaction();
	/// // Perform operations with the interaction
	/// ```
	///
	/// # Notes
	/// This method provides immutable access to the `CommandInteraction`
	/// member of the struct.
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	///
	/// Asynchronous function to fetch and generate embed content based on a user's AniList profile data.
	///
	/// # Description
	/// This function:
	/// - Retrieves the current execution context and accesses shared bot data.
	/// - Checks the command interaction for an optional "username" parameter.
	/// - If a username is provided, fetches user data from AniList for the given username.
	/// - If no username is provided, retrieves the user ID of the command invoker to query the database
	///   for a registered user and fetches user data from AniList based on the AniList ID in the database.
	/// - Fetches user details from AniList, formats the response, and generates content suitable for an embed.
	///
	/// # Returns
	/// - `Ok(Vec<EmbedContent<'_, '_>>)` containing the generated embed content if successful.
	/// - `Err` if any of the following occurs:
	///     - An error occurs during the database connection.
	///     - No user entry is found in the database for the invoker's user ID.
	///     - Errors occur while fetching or processing AniList user data.
	///
	/// # Errors
	/// - Returns an error in case:
	///     - The database query fails or no registered user matches the provided ID.
	///     - AniList API interaction (e.g., user fetching) fails.
	///     - The response formatting or content generation fails.
	///
	/// # Example
	/// ```ignore
	/// let contents = self.get_contents().await?;
	/// for content in contents {
	///     send_embed(content).await;
	/// }
	/// ```
	///
	/// # Dependencies
	/// - Requires `sea-orm` for database access.
	/// - Relies on AniList API interaction libraries for fetching user details.
	/// - Uses command interaction events to parse interaction options.
	///
	/// # Notes
	/// - The function returns early with the embed content if a username is explicitly provided.
	/// - Requires a valid configuration object (`config.db`) for database connection.
	/// - Uses a `RegisteredUser` table to look up stored AniList IDs if no username is provided.
	///
	/// # See Also
	/// - `get_user`: Function used to fetch AniList user data.
	/// - `user::user_content`: Function used to generate embed content for AniList user data.
	///
	async fn get_contents(&self) -> Result<Vec<EmbedContent<'_, '_>>> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();

		let config = bot_data.config.clone();

		let anilist_cache = bot_data.anilist_cache.clone();

		let map = get_option_map_string(command_interaction);

		let user = map.get(&FixedString::from_str_trunc("username"));

		// If the username is provided, fetch the user's data from AniList and send it as a response
		if let Some(value) = user {
			let data: User = get_user(value, anilist_cache.clone()).await?;

			let embed_content =
				user::user_content(command_interaction, data, config.db.clone()).await?;

			return Ok(embed_content);
		}

		let user_id = &command_interaction.user.id.to_string();

		let connection = sea_orm::Database::connect(get_url(config.db.clone())).await?;

		let row = RegisteredUser::find()
			.filter(Column::UserId.eq(user_id))
			.one(&connection)
			.await?;

		let user = row.ok_or(anyhow!("No user found"))?;

		// Fetch the user's data from AniList and send it as a response
		let data = get_user(user.anilist_id.to_string().as_str(), anilist_cache).await?;

		let embed_content =
			user::user_content(command_interaction, data, config.db.clone()).await?;

		Ok(embed_content)
	}
}

/// Asynchronously retrieves user data from the AniList API based on the provided input value.
///
/// This function accepts either a user ID (numerical value) or a username (string) and
/// attempts to fetch the associated user's data. It uses relevant GraphQL queries
/// (`UserQueryId` for IDs and `UserQuerySearch` for usernames) to retrieve the information.
///
/// # Arguments
///
/// * `value` - A string slice representing the user input. It can either be a user ID (if
///             it's parsable to an integer) or a username.
/// * `anilist_cache` - An `Arc`-wrapped, `RwLock`-protected cache (`Cache<String, String>`) to
///                     store and retrieve AniList API results.
///
/// # Returns
///
/// * `Result<User>` - Returns `Ok(User)` if the function successfully retrieves the user data
///   from the AniList API. If an error occurs, it returns an error wrapped within the `Result`.
///
/// # Errors
///
/// This function returns an error in cases such as:
/// * The input `value` cannot be successfully parsed as an integer (when treated as a user ID).
/// * The AniList API request fails.
/// * The API response fails to provide the expected user data (`data or user is None`).
///
/// # Examples
///
/// ```rust
/// use std::sync::Arc;
/// use tokio::sync::RwLock;
/// use your_crate_name::{get_user, Cache};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let cache = Arc::new(RwLock::new(Cache::new(100)));
///
///     // Fetch user by ID
///     let user_by_id = get_user("12345", cache.clone()).await?;
///
///     // Fetch user by username
///     let user_by_username = get_user("example_username", cache.clone()).await?;
///
///     Ok(())
/// }
/// ```
pub async fn get_user(
	value: &str, anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<User> {
	// If the value is a valid user ID, fetch the user's data by ID
	let user = if value.parse::<i32>().is_ok() {
		let id = value.parse::<i32>()?;

		let var = UserQueryIdVariables { id: Some(id) };

		let operation = UserQueryId::build(var);

		let data: GraphQlResponse<UserQueryId> =
			make_request_anilist(operation, false, anilist_cache).await?;

		data.data.unwrap().user.unwrap()
	} else {
		// If the value is not a valid user ID, fetch the user's data by username
		let var = UserQuerySearchVariables {
			search: Some(value),
		};

		let operation = UserQuerySearch::build(var);

		let data: GraphQlResponse<UserQuerySearch> =
			make_request_anilist(operation, false, anilist_cache).await?;

		data.data.unwrap().user.unwrap()
	};

	Ok(user)
}

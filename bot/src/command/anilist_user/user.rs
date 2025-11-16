//! The `UserCommand` struct is used to handle the execution of a user-related command.
//! This command fetches user information from AniList and optionally retrieves user-specific
//! data stored in the database.
//!
//! # Fields
//! - `ctx`: The Serenity context required to interact with the bot.
//! - `command_interaction`: Details about the command interaction being processed.
use crate::command::command::Command;
use crate::command::embed_content::EmbedsContents;
use crate::database::prelude::RegisteredUser;
use crate::database::registered_user::Column;
use crate::event_handler::BotData;
use crate::helper::get_option::command::get_option_map_string;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::run::anilist::user;
use crate::structure::run::anilist::user::{
    User, UserQueryId, UserQueryIdVariables, UserQuerySearch, UserQuerySearchVariables,
};
use crate::{get_url, impl_command};
use anyhow::{anyhow, Result};
use cynic::{GraphQlResponse, QueryBuilder};
use moka::future::Cache;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use small_fixed_array::FixedString;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::cache::CacheInterface;

#[derive(Clone)]
pub struct UserCommand {
    pub ctx: SerenityContext,
    pub command_interaction: CommandInteraction,
}

impl_command!(
	for UserCommand,
	get_contents = |self_: UserCommand| async move {
		let ctx = self_.get_ctx().clone();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self_.get_command_interaction().clone();

		let config = bot_data.config.clone();

	let anilist_cache = bot_data.anilist_cache;

		let map = get_option_map_string(&command_interaction);

		let user = map.get(&FixedString::from_str_trunc("username"));

		// If the username is provided, fetch the user's data from AniList and send it as a response
		if let Some(value) = user {
			let data: User = get_user(value, anilist_cache.clone()).await?;
			let db_connection = bot_data.db_connection.clone();

			let embed_content =
				user::user_content(command_interaction, data, db_connection).await?;

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
		let db_connection = bot_data.db_connection.clone();

		let embed_content = user::user_content(command_interaction, data, db_connection).await?;

		Ok(embed_content)
	}
);

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
	value: &str, anilist_cache: Arc<RwLock<CacheInterface>>,
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

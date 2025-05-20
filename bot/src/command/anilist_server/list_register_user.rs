//! `ListRegisterUser` is a struct that handles the functionality of listing registered users
//! in a Discord guild. It implements the `Command` trait to define specific behaviors
//! for interacting with Discord and retrieving necessary data.
use crate::command::command::{Command, CommandRun};
use crate::command::embed_content::{
	ButtonV1, CommandType, ComponentVersion, ComponentVersion1, EmbedContent, EmbedsContents,
};
use crate::constant::{ACTIVITY_LIST_LIMIT, MEMBER_LIST_LIMIT};
use crate::database::prelude::RegisteredUser;
use crate::database::registered_user::Column;
use crate::event_handler::BotData;
use crate::structure::message::anilist_server::list_register_user::load_localization_list_user;
use anyhow::{Result, anyhow};
use futures::StreamExt;
use futures::pin_mut;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::{ColumnTrait, DatabaseConnection};
use serenity::all::{
	CommandInteraction, Context as SerenityContext, CreateActionRow, CreateButton, PartialGuild,
	User, UserId,
};
use std::borrow::Cow;
use std::sync::Arc;
use tracing::trace;

/// A structure representing a user registration process in a system utilizing the Serenity framework.
/// This structure encapsulates the context and interaction required to handle a user's command.
///
/// # Fields
///
/// * `ctx` - Represents the `SerenityContext`, providing access to the bot's state and functionality,
///           such as interacting with Discord's API or accessing data shared across commands.
///
/// * `command_interaction` - Represents the `CommandInteraction`, which contains all the details
///                           about the user's interaction/command invocation, including the command
///                           name, options, and the interaction's associated metadata.
///
/// # Usage
/// This struct is designed to be used within the context of a Discord bot built with Serenity.
/// It combines the necessary context and command interaction data to facilitate processes such as
/// user
pub struct ListRegisterUser {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for ListRegisterUser {
	/// Retrieves a reference to the `SerenityContext` instance.
	///
	/// This method provides access to the `SerenityContext` associated
	/// with the current instance. The `SerenityContext` contains
	/// various components and shared data crucial for operating with
	/// the Serenity library, such as HTTP interaction, shard information,
	/// and cache.
	///
	/// # Returns
	///
	/// A reference to the `SerenityContext` stored in the current instance.
	///
	/// # Example
	///
	/// ```rust
	/// let context = instance.get_ctx();
	/// // Use the context for further operations
	/// ```
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` associated with the instance.
	///
	/// # Returns
	/// A reference to the `CommandInteraction` stored within the current instance.
	///
	/// # Examples
	/// ```
	/// let interaction = instance.get_command_interaction();
	/// // Use `interaction` as needed
	/// ```
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Asynchronously fetches and prepares the contents to be embedded in the response.
	///
	/// # Returns
	/// * `Result<EmbedsContents>` - A result containing the `EmbedsContents` on success or an error on failure.
	///
	/// This method performs the following steps:
	/// 1. Retrieves the application context and bot data.
	/// 2. Extracts the guild ID from the interaction; returns an error if absent.
	/// 3. Loads user localization for the given guild ID.
	/// 4. Fetches partial guild information with counts using the guild's ID.
	/// 5. Generates a description, count of users, and last user ID from the list of users.
	/// 6. Constructs an embed with the above-generated details.
	/// 7. If the number of users exceeds or equals the limit, adds a "Next" button for pagination.
	/// 8. Wraps the embed content and optional components into `EmbedsContents`.
	///
	/// # Errors
	/// Returns an error in the following situations:
	/// - The guild interaction fails to provide a guild ID.
	/// - Custom errors from localization loading.
	/// - Failures in fetching partial guild data or other external
	async fn get_contents(&self) -> Result<EmbedsContents> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();
		let config = bot_data.config.clone();
		let connection = bot_data.db_connection.clone();

		self.defer().await?;

		let guild_id = match command_interaction.guild_id {
			Some(id) => id,
			None => return Err(anyhow!("Failed to get the id of the guild")),
		};
		let list_user_localised =
			load_localization_list_user(guild_id.to_string(), config.db.clone()).await?;
		let guild = guild_id.to_partial_guild_with_counts(&ctx.http).await?;

		let (desc, len, last_id): (String, usize, Option<UserId>) =
			get_the_list(guild, ctx, None, connection).await?;
		let mut embed_content = EmbedContent::new(list_user_localised.title).description(desc);

		let action_row;
		if len >= MEMBER_LIST_LIMIT as usize {
			let buttons = vec![
				ButtonV1::new(list_user_localised.next)
					.custom_id(format!("user_{}_0", last_id.unwrap())),
			];
			let v1 = ComponentVersion1::buttons(buttons);
			action_row = Some(ComponentVersion::V1(v1));
		} else {
			action_row = None
		}

		let mut embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);
		if let Some(action_row) = action_row {
			embed_contents = embed_contents.action_row(action_row);
		}

		Ok(embed_contents)
	}
}

/// The `Data` struct serves as a container for user-related information and associated metadata.
///
/// # Fields
///
/// * `user` - Represents the user information stored as a `User` struct.
/// This typically includes user-specific details such as username, email, or ID.
///
/// * `anilist` - A `String` field that holds the AniList-related data, such as profile URL or ID.
/// This is useful to associate the user with their AniList account or data from the AniList API.
///
/// # Example
///
/// ```rust
/// struct User {
///     pub username: String,
/// }
///
/// let user = User {
///     username: String::from("example_user"),
/// };
///
/// let data = Data {
///     user,
///     anilist: String::
struct Data {
	pub user: User,
	pub anilist: String,
}

/// Asynchronously retrieves a formatted list of AniList user links for the members of a given Discord guild.
///
/// This function iterates through the members of the provided Discord guild, checks if they are registered
/// in the application's database, and retrieves their AniList IDs if applicable. The collected information
/// is formatted into a Markdown-compatible string of user links and returns additional metadata such as the
/// number of processed users and the ID of the last processed member.
///
/// # Arguments
///
/// * `guild` - A partial representation of the Discord guild (server) to process.
/// * `ctx` - A reference to the SerenityContext, used to interact with Discord's API.
/// * `last_id` - An optional identifier of the last processed user to continue from (useful for paginated requests).
/// * `connection` - A thread-safe reference to the application's database connection.
///
///
pub async fn get_the_list(
	guild: PartialGuild, ctx: &SerenityContext, last_id: Option<UserId>,
	connection: Arc<DatabaseConnection>,
) -> Result<(String, usize, Option<UserId>)> {
	let mut anilist_user = Vec::new();

	let mut last_id: Option<UserId> = last_id;

	let members = guild.id.members_iter(&ctx.http);
	pin_mut!(members);
	while let Some(result) = members.next().await {
		let member = match result {
			Ok(member) => member,
			Err(e) => return Err(anyhow!("Failed to get the members of the guild: {}", e)),
		};
		trace!("{:?}", member);
		last_id = Some(member.user.id);

		let user_id = member.user.id.to_string();

		let row = match RegisteredUser::find()
			.filter(Column::UserId.eq(user_id.clone()))
			.one(&*connection)
			.await?
		{
			Some(row) => row,
			None => continue,
		};
		trace!("{:?}", row);

		let user_data = row;

		let data = Data {
			user: member.user,
			anilist: user_data.anilist_id.to_string(),
		};

		anilist_user.push(data)
	}

	let user_links: Vec<String> = anilist_user
		.iter()
		.map(|data| {
			format!(
				"[{}](<https://anilist.co/user/{}>)",
				data.user.name, data.anilist
			)
		})
		.collect();

	let joined_string = user_links.join("\n");

	Ok((joined_string, anilist_user.len(), last_id))
}

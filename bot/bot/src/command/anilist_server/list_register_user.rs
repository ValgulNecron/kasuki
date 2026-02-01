//! `ListRegisterUser` is a struct that handles the functionality of listing registered users
//! in a Discord guild. It implements the `Command` trait to define specific behaviors
//! for interacting with Discord and retrieving necessary data.
use crate::command::command::{Command, CommandRun};
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::constant::MEMBER_LIST_LIMIT;
use crate::event_handler::BotData;
use crate::impl_command;
use anyhow::{anyhow, Result};
use fluent_templates::Loader;
use futures::pin_mut;
use futures::StreamExt;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::{ColumnTrait, DatabaseConnection};
use serenity::all::{CommandInteraction, Context as SerenityContext, PartialGuild, User, UserId};
use shared::database::prelude::RegisteredUser;
use shared::database::registered_user::Column;
use shared::helper::get_guild_lang::get_guild_language;
use shared::localization::USABLE_LOCALES;
use std::str::FromStr;
use std::sync::Arc;
use tracing::trace;
use unic_langid::LanguageIdentifier;

#[derive(Clone)]
pub struct ListRegisterUser {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl_command!(
	for ListRegisterUser,
	get_contents = |self_: ListRegisterUser| async move {
		let ctx = self_.get_ctx().clone();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self_.get_command_interaction().clone();
		let _config = bot_data.config.clone();
		let connection = bot_data.db_connection.clone();

		self_.defer().await?;

		let guild_id = match command_interaction.guild_id {
			Some(id) => id,
			None => return Err(anyhow!("Failed to get the id of the guild")),
		};
		let db_connection = bot_data.db_connection.clone();

		let lang = get_guild_language(guild_id.to_string(), db_connection).await;
		let lang_code = match lang.as_str() {
			"jp" => "ja",
			"en" => "en-US",
			other => other,
		};
		let lang_id = LanguageIdentifier::from_str(lang_code)
			.unwrap_or_else(|_| LanguageIdentifier::from_str("en-US").unwrap());
		let title = USABLE_LOCALES.lookup(&lang_id, "anilist_server_list_register_user-title");

		let guild = guild_id.to_partial_guild_with_counts(&ctx.http).await?;

		let (desc, len, _last_id): (String, usize, Option<UserId>) =
			get_the_list(guild, &ctx, None, connection).await?;
		let embed_content = EmbedContent::new(title).description(desc);

		let action_row;
		if len >= MEMBER_LIST_LIMIT as usize {
			action_row = None
		} else {
			action_row = None
		}

		let mut embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);
		if let Some(action_row) = action_row {
			embed_contents = embed_contents.action_row(action_row);
		}

		Ok(embed_contents)
	}
);

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

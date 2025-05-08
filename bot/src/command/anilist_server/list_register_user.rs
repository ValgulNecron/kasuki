//! `ListRegisterUser` is a structure that implements the [`Command`] trait and is responsible for
//! listing registered users in a Discord guild, fetching their associated AniList user IDs,
//! and displaying them via Discord Slash commands.
//!
//! This implementation fetches guild members, queries the associated registered users in the database,
//! and formats the data to be shown in an embed response.
//!
//! # Fields
//!
//! * `ctx`: The [`SerenityContext`] instance, representing the Discord bot's context.
//! * `command
use crate::command::command_trait::{Command, CommandRun, EmbedContent, EmbedType};
use crate::constant::MEMBER_LIST_LIMIT;
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

/// A structure that encapsulates context and interaction details for listing registered users.
///
/// The `ListRegisterUser` struct is used to handle command interactions within the Serenity framework.
/// It provides access to the command interaction data and the bot's context, making it useful for
/// implementing functionality that involves listing
pub struct ListRegisterUser {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for ListRegisterUser {
	/// Retrieves a reference to the `SerenityContext` associated with the current instance.
	///
	/// # Returns
	/// A reference to the `SerenityContext` (`&SerenityContext`).
	///
	/// # Example
	/// ```rust
	/// let context = instance.get_ctx();
	/// // You can now use the context to interact with the Discord API.
	/// ```
	///
	/// This method provides access to
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` instance associated with the current object.
	///
	/// # Returns
	/// A reference to the `CommandInteraction` instance stored within the object.
	///
	/// # Example
	/// ```rust
	///
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Asynchronously retrieves a list of `EmbedContent` with user-related information.
	///
	/// This function interacts with a Discord guild to fetch a list of users and their associated data.
	/// It defers the initial response to indicate processing and fetches necessary information
	/// like guild configuration, localization data, and user lists.
	///
	/// # Returns:
	/// -
	async fn get_contents(&self) -> Result<Vec<EmbedContent<'_, '_>>> {
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
		let list_user_localised = load_localization_list_user(guild_id.to_string(), config.db.clone()).await?;
		let guild = guild_id.to_partial_guild_with_counts(&ctx.http).await?;

		let (desc, len, last_id): (String, usize, Option<UserId>) =
			get_the_list(guild, ctx, None, connection).await?;
		let mut embed_content = EmbedContent::new(list_user_localised.title)
			.description(desc)
			.command_type(EmbedType::Followup);
		if len >= MEMBER_LIST_LIMIT as usize {
			embed_content.action_row = vec![CreateActionRow::Buttons(Cow::from(vec![
				CreateButton::new(format!("user_{}_0", last_id.unwrap()))
					.label(list_user_localised.next),
			]))];
		}

		Ok(vec![embed_content]).await
	}
}

/// A struct representing a collection of data associated with a user.
///
/// # Fields
/// - `user` (`User`): A structure representing the user data.
/// - `anilist` (`String`): A string representing the AniList username or identifier corresponding to the user.
struct Data {
	pub user: User,
	pub anilist: String,
}

/// Asynchronously retrieves a list of AniList-linked users from a partial guild and returns them in a formatted string.
///
/// # Parameters
/// - `guild`: The partial guild object representing the guild from which to fetch members.
/// - `ctx`: A
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

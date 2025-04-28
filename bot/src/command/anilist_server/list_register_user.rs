use std::borrow::Cow;
use std::sync::Arc;
use crate::command::command_trait::{Command, Embed, EmbedContent, EmbedType, SlashCommand};
use crate::constant::{MEMBER_LIST_LIMIT};
use crate::database::prelude::RegisteredUser;
use crate::database::registered_user::Column;
use crate::event_handler::BotData;
use crate::structure::message::anilist_server::list_register_user::load_localization_list_user;
use anyhow::{Result, anyhow};
use futures::StreamExt;
use sea_orm::{ColumnTrait, DatabaseConnection};
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use serenity::all::{CommandInteraction, Context as SerenityContext, CreateActionRow, CreateButton, PartialGuild, User, UserId};
use tracing::trace;
use futures::{pin_mut};

pub struct ListRegisterUser {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for ListRegisterUser {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for ListRegisterUser {
	async fn run_slash(&self) -> Result<()> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();
		let config = bot_data.config.clone();
		let connection = bot_data.db_connection.clone();

		let guild_id = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};

		// Load the localized text for the list user command
		let list_user_localised = load_localization_list_user(guild_id, config.db.clone()).await?;

		// Retrieve the guild from the guild ID
		let guild_id = match command_interaction.guild_id {
			Some(id) => id,
			None => return Err(anyhow!("Failed to get the id of the guild")),
		};

		let guild = guild_id.to_partial_guild_with_counts(&ctx.http).await?;

		self.defer().await?;

		let (desc, len, last_id): (String, usize, Option<UserId>) =
			get_the_list(guild, ctx, None, connection).await?;

		let mut content = EmbedContent::new(list_user_localised.title)
			.description(desc)
			.command_type(EmbedType::Followup);

		if len >= MEMBER_LIST_LIMIT as usize {
			content.action_row = vec![CreateActionRow::Buttons(Cow::from(vec![
				CreateButton::new(format!("user_{}_0", last_id.unwrap()))
					.label(list_user_localised.next),
			]))];
		}
		self.send_embed(vec![content]).await
	}
}

struct Data {
	pub user: User,
	pub anilist: String,
}

pub async fn get_the_list(
	guild: PartialGuild, ctx: &SerenityContext, last_id: Option<UserId>, connection: Arc<DatabaseConnection>,
) -> Result<(String, usize, Option<UserId>)> {
	let mut anilist_user = Vec::new();

	let mut last_id: Option<UserId> = last_id;

	let members = guild
		.id
		.members_iter(
			&ctx.http,
		);
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

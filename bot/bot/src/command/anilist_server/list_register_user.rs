use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::constant::MEMBER_LIST_LIMIT;
use anyhow::{anyhow, Result};
use fluent_templates::Loader;
use futures::pin_mut;
use futures::StreamExt;
use kasuki_macros::slash_command;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::{ColumnTrait, DatabaseConnection};
use serenity::all::{CommandInteraction, Context as SerenityContext, PartialGuild, User, UserId};
use shared::database::prelude::RegisteredUser;
use shared::database::registered_user::Column;
use shared::localization::USABLE_LOCALES;
use std::sync::Arc;
use tracing::trace;

#[slash_command(
	name = "list_user", desc = "Get the list of registered user.", command_type = ChatInput,
	contexts = [Guild],
	install_contexts = [Guild],
)]
async fn list_register_user_command(self_: ListRegisterUser) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);

	let guild_id = cx
		.command_interaction
		.guild_id
		.ok_or(anyhow!("Failed to get the id of the guild"))?;

	let lang_id = cx.lang_id().await;
	let title = USABLE_LOCALES.lookup(&lang_id, "anilist_server_list_register_user-title");

	let guild = guild_id
		.to_partial_guild_with_counts(&cx.ctx.http)
		.await?;

	let (desc, len, _last_id): (String, usize, Option<UserId>) =
		get_the_list(guild, &cx.ctx, None, cx.db.clone()).await?;
	let embed_content = EmbedContent::new(title).description(desc);

	let action_row;
	if len >= MEMBER_LIST_LIMIT as usize {
		action_row = None
	} else {
		action_row = None
	}

	let mut embed_contents = EmbedsContents::new(vec![embed_content]);
	if let Some(action_row) = action_row {
		embed_contents = embed_contents.action_row(action_row);
	}

	Ok(embed_contents)
}

struct Data {
	pub user: User,
	pub anilist: String,
}

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

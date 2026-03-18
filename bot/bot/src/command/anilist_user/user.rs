use crate::command::context::CommandContext;
use crate::helper::get_option::command::get_option_map_string;
use crate::structure::run::anilist::user;
use crate::structure::run::anilist::user::get_user;
use anyhow::{anyhow, Result};
use kasuki_macros::slash_command;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::database::prelude::RegisteredUser;
use shared::database::registered_user::Column;
use small_fixed_array::FixedString;

#[slash_command(
	name = "anilist_user", desc = "Info of an user on AniList.", command_type = ChatInput,
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
	args = [(name = "username", desc = "Username of the user you want to check.", arg_type = String, required = false, autocomplete = true)],
)]
async fn user_command(self_: UserCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);
	let anilist_cache = cx.anilist_cache.clone();

	let map = get_option_map_string(&cx.command_interaction);

	let lang_id = cx.lang_id().await;
	let user = map.get(&FixedString::from_str_trunc("username"));

	if let Some(value) = user {
		let data = get_user(value, anilist_cache.clone()).await?;

		let embed_content = user::user_content(data, &lang_id).await?;

		return Ok(embed_content);
	}

	let user_id = &cx.command_interaction.user.id.to_string();

	let row = RegisteredUser::find()
		.filter(Column::UserId.eq(user_id))
		.one(&*cx.db)
		.await?;

	let user = row.ok_or(anyhow!("No user found"))?;

	let data = get_user(&user.anilist_id.to_string(), anilist_cache).await?;
	let embed_content = user::user_content(data, &lang_id).await?;

	Ok(embed_content)
}

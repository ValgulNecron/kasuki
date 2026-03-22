use anyhow::anyhow;

use fluent_templates::Loader;
use kasuki_macros::slash_command;
use sea_orm::ActiveValue::Set;
use sea_orm::EntityTrait;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::fluent_args;
use shared::localization::USABLE_LOCALES;
use small_fixed_array::FixedString;

use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::helper::get_option::command::get_option_map_string;
use crate::structure::run::anilist::user::{get_color, get_user, get_user_url, User};
use shared::database::prelude::RegisteredUser;
use shared::database::registered_user::{ActiveModel, Column};

#[slash_command(
	name = "register", desc = "Register your username on AniList.", command_type = ChatInput,
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
	args = [(name = "username", desc = "Username you want to register.", arg_type = String, required = true, autocomplete = true)],
)]
async fn register_command(self_: RegisterCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);
	let anilist_cache = cx.anilist_cache.clone();
	let connection = cx.db.clone();

	let map = get_option_map_string(&cx.command_interaction);

	let value = map
		.get(&FixedString::from_str_trunc("username"))
		.ok_or(anyhow!("No username provided"))?;

	let user_data: User = get_user(value, anilist_cache).await?;

	let lang_id = cx.lang_id().await;

	let user_id = &cx.command_interaction.user.id.to_string();

	let username = &cx.command_interaction.user.name;

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

	let args = fluent_args!(
		"user" => username.as_str(),
		"id" => user_id.as_str(),
		"anilist" => user_data.name.clone(),
	);

	let desc = USABLE_LOCALES.lookup_with_args(&lang_id, "anilist_user_register-desc", &args);

	let embed_content = EmbedContent::new(user_data.clone().name)
		.description(desc)
		.thumbnail(user_data.clone().avatar.unwrap().large.unwrap())
		.url(get_user_url(&user_data.id))
		.colour(get_color(user_data.clone()));

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}

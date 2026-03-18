use crate::command::admin::anilist::add_activity::{get_minimal_anime_media, get_name};
use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::helper::get_option::subcommand_group::get_option_map_string_subcommand_group;
use anyhow::{anyhow, Result};
use fluent_templates::fluent_bundle::FluentValue;
use fluent_templates::Loader;
use kasuki_macros::slash_command;
use sea_orm::ColumnTrait;
use sea_orm::DatabaseConnection;
use sea_orm::{EntityTrait, ModelTrait, QueryFilter};
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::database::prelude::ActivityData;
use shared::localization::USABLE_LOCALES;
use std::borrow::Cow;
use std::collections::HashMap;

#[slash_command(
	name = "delete_anime_activity", desc = "Delete an anime activity.",
	command_type = SubCommandGroup(parent = "admin", group = "anilist"),
	args = [(name = "anime_name", desc = "Name of the anime you want to delete as an activity.", arg_type = String, required = true, autocomplete = true)],
)]
async fn delete_activity_command(self_: DeleteActivityCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);

	let map = get_option_map_string_subcommand_group(&cx.command_interaction);
	let anime = map.get("anime_name").cloned().unwrap_or(String::new());

	let lang_id = cx.lang_id().await;
	let media = get_minimal_anime_media(anime.to_string(), cx.anilist_cache.clone()).await?;
	let anime_id = media.id;

	remove_activity(cx.guild_id.as_str(), &anime_id, &cx.db).await?;

	let title = media
		.title
		.ok_or(anyhow!("Anime with id {} not found", anime_id))?;
	let anime_name = get_name(title);

	let url = format!("https://anilist.co/anime/{}", anime_id);

	let mut args = HashMap::new();
	args.insert(
		Cow::Borrowed("anime"),
		FluentValue::from(anime_name.as_str()),
	);

	let embed_content =
		EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "admin_anilist_delete_activity-success"))
			.description(USABLE_LOCALES.lookup_with_args(
				&lang_id,
				"admin_anilist_delete_activity-success_desc",
				&args,
			))
			.url(url);

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}

async fn remove_activity(guild_id: &str, anime_id: &i32, db: &DatabaseConnection) -> Result<()> {
	let activity = ActivityData::find()
		.filter(shared::database::activity_data::Column::ServerId.eq(guild_id))
		.filter(shared::database::activity_data::Column::AnimeId.eq(*anime_id))
		.one(db)
		.await?
		.ok_or(anyhow!("Anime with id {} not found", anime_id))?;

	activity.delete(db).await?;

	Ok(())
}

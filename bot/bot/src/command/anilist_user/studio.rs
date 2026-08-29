use anyhow::anyhow;

use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::helper::get_option::command::get_option_map_string;
use crate::structure::run::anilist::studio::{
	StudioQuerryId, StudioQuerryIdVariables, StudioQuerrySearch, StudioQuerrySearchVariables,
};
use cynic::{GraphQlResponse, QueryBuilder};
use fluent_templates::Loader;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::anilist::make_request::make_request_anilist;
use shared::fluent_args;
use shared::localization::USABLE_LOCALES;
use small_fixed_array::FixedString;

#[slash_command(
	name = "studio", desc = "Info of a studio.", command_type = ChatInput,
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
	args = [(name = "studio", desc = "Name of the studio you want to check.", arg_type = String, required = true, autocomplete = true)],
)]
async fn studio_command(self_: StudioCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);
	let anilist_cache = cx.anilist_cache.clone();

	let map = get_option_map_string(&cx.command_interaction);

	let value = map
		.get(&FixedString::from_str_trunc("studio"))
		.ok_or(anyhow!("No studio specified"))?;

	let studio = if value.parse::<i32>().is_ok() {
		let id = value.parse::<i32>()?;

		let var = StudioQuerryIdVariables { id: Some(id) };

		let operation = StudioQuerryId::build(var);

		let data: GraphQlResponse<StudioQuerryId> =
			make_request_anilist(operation, true, anilist_cache).await?;

		data.data.unwrap().studio.unwrap()
	} else {
		let var = StudioQuerrySearchVariables {
			search: Some(value.as_str()),
		};

		let operation = StudioQuerrySearch::build(var);

		let data: GraphQlResponse<StudioQuerrySearch> =
			make_request_anilist(operation, true, anilist_cache).await?;

		data.data.unwrap().studio.unwrap()
	};

	let lang_id = cx.lang_id().await;

	let mut content = String::new();

	for m in studio.media.unwrap().nodes.unwrap() {
		let m = m.unwrap();

		let title = m.title.unwrap().clone();

		let rj = title.romaji.unwrap_or_default();

		let en = title.user_preferred.unwrap_or_default();

		let text = format!("[{}/{}]({})", rj, en, m.site_url.unwrap_or_default());

		content.push_str(text.as_str());

		content.push('\n')
	}

	let args = fluent_args!(
		"id" => studio.id.to_string(),
		"fav" => studio.favourites.unwrap_or_default().to_string(),
		"animation" => studio.is_animation_studio.to_string(),
		"list" => content.as_str(),
	);

	let desc = USABLE_LOCALES.lookup_with_args(&lang_id, "anilist_user_studio-desc", &args);

	let name = studio.name;

	let embed_content = EmbedContent::new(name)
		.description(desc)
		.url(studio.site_url.unwrap_or_default());

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}

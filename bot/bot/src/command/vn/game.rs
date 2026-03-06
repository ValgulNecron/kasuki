use crate::command::command::CommandRun;
use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use kasuki_macros::slash_command;
use markdown_converter::vndb::convert_vndb_markdown;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::{Loader, USABLE_LOCALES};
use shared::vndb::game::get_vn;
use tracing::trace;

#[slash_command(
	name = "game", desc = "Get info of a visual novel.",
	command_type = SubCommand(parent = "vn"),
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
	args = [(name = "title", desc = "Title of the visual novel.", arg_type = String, required = true, autocomplete = true)],
)]
async fn vn_game_command(self_: VnGameCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);
	let vndb_cache = cx.vndb_cache.clone();

	let map = get_option_map_string_subcommand(&cx.command_interaction);

	trace!("{:?}", map);

	let game = map
		.get(&String::from("title"))
		.cloned()
		.unwrap_or(String::new());

	let lang_id = cx.lang_id().await;

	let vn = get_vn(game.clone(), vndb_cache, &cx.bot_data.http_client).await?;

	let vn = vn.results[0].clone();

	let mut fields = vec![];

	if let Some(released) = vn.released {
		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "vn_game-released"),
			released,
			true,
		));
	}

	let platforms = vn
		.platforms
		.iter()
		.take(10)
		.cloned()
		.collect::<Vec<String>>()
		.join(", ");

	if !platforms.is_empty() {
		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "vn_game-platforms"),
			platforms,
			true,
		));
	}

	if let Some(playtime) = vn.length_minutes {
		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "vn_game-playtime"),
			playtime.to_string(),
			true,
		));
	}

	let tags = vn
		.tags
		.iter()
		.map(|tag| tag.name.clone())
		.take(10)
		.collect::<Vec<String>>()
		.join(", ");

	if !tags.is_empty() {
		fields.push((USABLE_LOCALES.lookup(&lang_id, "vn_game-tags"), tags, true));
	}

	let developers = vn
		.developers
		.iter()
		.map(|dev| dev.name.clone())
		.take(10)
		.collect::<Vec<String>>()
		.join(", ");

	if !developers.is_empty() {
		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "vn_game-developers"),
			developers,
			true,
		));
	}

	let staff = vn
		.staff
		.iter()
		.map(|staff| staff.name.clone())
		.take(10)
		.collect::<Vec<String>>()
		.join(", ");

	if !staff.is_empty() {
		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "vn_game-staff"),
			staff,
			true,
		));
	}

	let characters = vn
		.va
		.iter()
		.map(|va| va.character.name.clone())
		.take(10)
		.collect::<Vec<String>>()
		.join(", ");

	if !characters.is_empty() {
		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "vn_game-characters"),
			characters,
			true,
		));
	}
	let vn_desc = vn.description.clone().unwrap_or_default();

	let mut embed_content = EmbedContent::new(vn.title.clone())
		.description(convert_vndb_markdown(&vn_desc).to_string())
		.fields(fields)
		.url(format!("https://vndb.org/{}", vn.id));

	let sexual = match vn.image.clone() {
		Some(image) => image.sexual,
		None => 2.0,
	};

	let violence = match vn.image.clone() {
		Some(image) => image.violence,
		None => 2.0,
	};

	let url: Option<String> = match vn.image {
		Some(image) => Some(image.url.clone()),
		None => None,
	};

	if (sexual <= 1.5) && (violence <= 1.0) {
		if let Some(url) = url {
			embed_content = embed_content.images_url(url);
		}
	}

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}

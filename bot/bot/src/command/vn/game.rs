use crate::command::command::CommandRun;
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::helper::vndbapi::game::get_vn;
use kasuki_macros::slash_command;
use markdown_converter::vndb::convert_vndb_markdown;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::{get_language_identifier, Loader, USABLE_LOCALES};
use tracing::trace;

#[slash_command(
	name = "game", desc = "Get info of a visual novel.",
	command_type = SubCommand(parent = "vn"),
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
	args = [(name = "title", desc = "Title of the visual novel.", arg_type = String, required = true, autocomplete = true)],
)]
async fn vn_game_command(self_: VnGameCommand) -> Result<EmbedsContents<'_>> {
	self_.defer().await?;
	let ctx = self_.get_ctx();
	let bot_data = ctx.data::<BotData>().clone();
	let command_interaction = self_.get_command_interaction();
	let db_connection = bot_data.db_connection.clone();
	let vndb_cache = bot_data.vndb_cache.clone();

	let guild_id = match command_interaction.guild_id {
		Some(id) => id.to_string(),
		None => String::from("0"),
	};

	let map = get_option_map_string_subcommand(command_interaction);

	trace!("{:?}", map);

	let game = map
		.get(&String::from("title"))
		.cloned()
		.unwrap_or(String::new());

	let lang_id = get_language_identifier(guild_id, db_connection).await;

	let vn = get_vn(game.clone(), vndb_cache).await?;

	let vn = vn.results[0].clone();

	let mut fields = vec![];

	if let Some(released) = vn.released {
		fields.push((USABLE_LOCALES.lookup(&lang_id, "vn_game-released"), released, true));
	}

	let platforms = vn
		.platforms
		.iter()
		.take(10)
		.cloned()
		.collect::<Vec<String>>()
		.join(", ");

	if !platforms.is_empty() {
		fields.push((USABLE_LOCALES.lookup(&lang_id, "vn_game-platforms"), platforms, true));
	}

	if let Some(playtime) = vn.length_minutes {
		fields.push((USABLE_LOCALES.lookup(&lang_id, "vn_game-playtime"), playtime.to_string(), true));
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
		fields.push((USABLE_LOCALES.lookup(&lang_id, "vn_game-developers"), developers, true));
	}

	let staff = vn
		.staff
		.iter()
		.map(|staff| staff.name.clone())
		.take(10)
		.collect::<Vec<String>>()
		.join(", ");

	if !staff.is_empty() {
		fields.push((USABLE_LOCALES.lookup(&lang_id, "vn_game-staff"), staff, true));
	}

	let characters = vn
		.va
		.iter()
		.map(|va| va.character.name.clone())
		.take(10)
		.collect::<Vec<String>>()
		.join(", ");

	if !characters.is_empty() {
		fields.push((USABLE_LOCALES.lookup(&lang_id, "vn_game-characters"), characters, true));
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

	let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

	Ok(embed_contents)
}

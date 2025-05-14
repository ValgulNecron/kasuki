use std::sync::Arc;

use crate::command::command::{Command, CommandRun, EmbedContent, SlashCommand};
use crate::config::Config;
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::helper::vndbapi::game::get_vn;
use crate::structure::message::vn::game::load_localization_game;
use anyhow::Result;
use markdown_converter::vndb::convert_vndb_markdown;
use moka::future::Cache;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use tokio::sync::RwLock;
use tracing::trace;

pub struct VnGameCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for VnGameCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for VnGameCommand {
	async fn run_slash(&self) -> Result<()> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();
		let config = bot_data.config.clone();
		let vndb_cache = bot_data.vndb_cache.clone();

		let content = get_content(command_interaction, config, vndb_cache).await?;

		self.send_embed(vec![content]).await
	}
}

async fn get_content(
	command_interaction: &CommandInteraction, config: Arc<Config>,
	vndb_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<EmbedContent<'static, 'static>> {
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

	let game_localised = load_localization_game(guild_id, config.db.clone()).await?;

	let vn = get_vn(game.clone(), vndb_cache).await?;

	let vn = vn.results[0].clone();

	let mut fields = vec![];

	if let Some(released) = vn.released {
		fields.push((game_localised.released.clone(), released, true));
	}

	let platforms = vn
		.platforms
		.iter()
		.take(10)
		.cloned()
		.collect::<Vec<String>>()
		.join(", ");

	if !platforms.is_empty() {
		fields.push((game_localised.platforms.clone(), platforms, true));
	}

	if let Some(playtime) = vn.length_minutes {
		fields.push((game_localised.playtime.clone(), playtime.to_string(), true));
	}

	let tags = vn
		.tags
		.iter()
		.map(|tag| tag.name.clone())
		.take(10)
		.collect::<Vec<String>>()
		.join(", ");

	if !tags.is_empty() {
		fields.push((game_localised.tags.clone(), tags, true));
	}

	let developers = vn
		.developers
		.iter()
		.map(|dev| dev.name.clone())
		.take(10)
		.collect::<Vec<String>>()
		.join(", ");

	if !developers.is_empty() {
		fields.push((game_localised.developers.clone(), developers, true));
	}

	let staff = vn
		.staff
		.iter()
		.map(|staff| staff.name.clone())
		.take(10)
		.collect::<Vec<String>>()
		.join(", ");

	if !staff.is_empty() {
		fields.push((game_localised.staff.clone(), staff, true));
	}

	let characters = vn
		.va
		.iter()
		.map(|va| va.character.name.clone())
		.take(10)
		.collect::<Vec<String>>()
		.join(", ");

	if !characters.is_empty() {
		fields.push((game_localised.characters.clone(), characters, true));
	}
	let vn_desc = vn.description.clone().unwrap_or_default();

	let mut content = EmbedContent::new(vn.title.clone())
		.description(convert_vndb_markdown(&vn_desc).to_string())
		.fields(fields)
		.url(Some(format!("https://vndb.org/{}", vn.id)));

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
			content = content.images_url(Some(url));
		}
	}

	Ok(content)
}

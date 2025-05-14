use crate::command::command::{Command, CommandRun, EmbedContent, SlashCommand};
use crate::config::Config;
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::helper::vndbapi::producer::get_producer;
use crate::structure::message::vn::producer::load_localization_producer;
use anyhow::Result;
use markdown_converter::vndb::convert_vndb_markdown;
use moka::future::Cache;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::trace;

pub struct VnProducerCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for VnProducerCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for VnProducerCommand {
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

	let producer = map
		.get(&String::from("name"))
		.cloned()
		.unwrap_or(String::new());
	let db_config = config.db.clone();

	let producer_localised = load_localization_producer(guild_id, db_config.clone()).await?;

	let producer = get_producer(producer.clone(), vndb_cache.clone()).await?;

	let producer = producer.results[0].clone();

	let mut fields = vec![];

	if let Some(lang) = producer.lang {
		fields.push((producer_localised.lang.clone(), lang, true));
	}

	if let Some(aliases) = producer.aliases {
		let aliases = aliases
			.into_iter()
			.take(10)
			.collect::<Vec<String>>()
			.join(", ");

		fields.push((producer_localised.aliases.clone(), aliases, true));
	}

	if let Some(results_type) = producer.results_type {
		fields.push((
			producer_localised.prod_type.clone(),
			results_type.to_string(),
			true,
		));
	}
	let prod_desc = producer.description.clone().unwrap_or_default();

	let content = EmbedContent::new(producer.name.clone())
		.description(String::from(convert_vndb_markdown(&prod_desc)))
		.fields(fields)
		.url(Some(format!("https://vndb.org/{}", producer.id)));

	Ok(content)
}

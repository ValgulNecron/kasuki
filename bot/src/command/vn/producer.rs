use crate::command::command_trait::{Command, SlashCommand};
use crate::event_handler::BotData;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::helper::vndbapi::producer::get_producer;
use crate::structure::message::vn::producer::load_localization_producer;
use anyhow::Result;
use markdown_converter::vndb::convert_vndb_markdown;
use serenity::all::{
	CommandInteraction, Context as SerenityContext, CreateInteractionResponse,
	CreateInteractionResponseMessage,
};
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
		send_embed(&self.ctx, &self.command_interaction).await
	}
}

async fn send_embed(ctx: &SerenityContext, command_interaction: &CommandInteraction) -> Result<()> {
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
	let bot_data = ctx.data::<BotData>().clone();

	let producer_localised =
		load_localization_producer(guild_id, bot_data.config.db.clone()).await?;

	let producer = get_producer(producer.clone(), bot_data.vndb_cache.clone()).await?;

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

	let builder_embed = get_default_embed(None)
		.description(convert_vndb_markdown(&prod_desc))
		.fields(fields)
		.title(producer.name.clone())
		.url(format!("https://vndb.org/{}", producer.id));

	let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

	let builder = CreateInteractionResponse::Message(builder_message);

	command_interaction
		.create_response(&ctx.http, builder)
		.await?;

	Ok(())
}

use crate::command::command::CommandRun;
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::helper::vndbapi::producer::get_producer;
use kasuki_macros::slash_command;
use markdown_converter::vndb::convert_vndb_markdown;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::{get_language_identifier, Loader, USABLE_LOCALES};
use tracing::trace;

#[slash_command(
	name = "producer", desc = "Get info of a VN producer.",
	command_type = SubCommand(parent = "vn"),
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
	args = [(name = "name", desc = "Name of the producer.", arg_type = String, required = true, autocomplete = true)],
)]
async fn vn_producer_command(self_: VnProducerCommand) -> Result<EmbedsContents<'_>> {
	self_.defer().await?;
	let ctx = self_.get_ctx();
	let bot_data = ctx.data::<BotData>().clone();
	let command_interaction = self_.get_command_interaction();
	let vndb_cache = bot_data.vndb_cache.clone();
	let db_connection = bot_data.db_connection.clone();

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

	let lang_id = get_language_identifier(guild_id, db_connection).await;

	let producer = get_producer(producer.clone(), vndb_cache.clone()).await?;

	let producer = producer.results[0].clone();

	let mut fields = vec![];

	if let Some(lang) = producer.lang {
		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "vn_producer-lang"),
			lang,
			true,
		));
	}

	if let Some(aliases) = producer.aliases {
		let aliases = aliases
			.into_iter()
			.take(10)
			.collect::<Vec<String>>()
			.join(", ");

		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "vn_producer-aliases"),
			aliases,
			true,
		));
	}

	if let Some(results_type) = producer.results_type {
		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "vn_producer-prod_type"),
			results_type.to_string(),
			true,
		));
	}
	let prod_desc = producer.description.clone().unwrap_or_default();

	let embed_content = EmbedContent::new(producer.name.clone())
		.description(String::from(convert_vndb_markdown(&prod_desc)))
		.fields(fields)
		.url(format!("https://vndb.org/{}", producer.id));

	let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

	Ok(embed_contents)
}

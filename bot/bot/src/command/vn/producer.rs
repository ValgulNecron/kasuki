use crate::command::command::CommandRun;
use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use kasuki_macros::slash_command;
use markdown_converter::vndb::convert_vndb_markdown;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::{Loader, USABLE_LOCALES};
use shared::vndb::producer::get_producer;
use tracing::trace;

#[slash_command(
	name = "producer", desc = "Get info of a VN producer.",
	command_type = SubCommand(parent = "vn"),
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
	args = [(name = "name", desc = "Name of the producer.", arg_type = String, required = true, autocomplete = true)],
)]
async fn vn_producer_command(self_: VnProducerCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);
	let vndb_cache = cx.vndb_cache.clone();

	let map = get_option_map_string_subcommand(&cx.command_interaction);

	trace!("{:?}", map);

	let producer = map
		.get(&String::from("name"))
		.cloned()
		.unwrap_or(String::new());

	let lang_id = cx.lang_id().await;

	let producer =
		get_producer(producer.clone(), vndb_cache.clone(), &cx.bot_data.http_client).await?;

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

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}

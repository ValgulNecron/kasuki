use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::command::prenium_command::{PremiumCommand, PremiumCommandType};
use crate::constant::DEFAULT_STRING;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use anyhow::anyhow;
use fluent_templates::Loader;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::USABLE_LOCALES;

#[slash_command(
	name = "question", desc = "Ask a question and get the response (this is not a chat it has no context).",
	command_type = SubCommand(parent = "ai"),
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
	extra_fields = [command_name: String = full_command_name.to_string()],
	args = [
		(name = "prompt", desc = "What you want to ask.", arg_type = String, required = true, autocomplete = false)
	],
)]
async fn question_command(self_: QuestionCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);
	let config = &cx.config;
	let lang_id = cx.lang_id().await;

	if self_
		.check_hourly_limit(
			self_.command_name.clone(),
			&cx.bot_data,
			PremiumCommandType::AIQuestion,
		)
		.await?
	{
		let error_msg = USABLE_LOCALES.lookup(&lang_id, "ai_question-hourly_limit");
		return Err(anyhow!(error_msg));
	}

	let map = get_option_map_string_subcommand(&cx.command_interaction);
	let prompt = map
		.get("prompt")
		.map(String::as_str)
		.unwrap_or(DEFAULT_STRING);

	let api_key = config
		.ai
		.question
		.ai_question_token
		.clone()
		.unwrap_or_default();
	let api_base_url = config
		.ai
		.question
		.ai_question_base_url
		.clone()
		.unwrap_or_default();
	let model = config
		.ai
		.question
		.ai_question_model
		.clone()
		.unwrap_or_default();

	let text = shared::service::ai::question(
		prompt,
		&api_key,
		&api_base_url,
		&model,
		&cx.bot_data.http_client,
	)
	.await?;

	let embed_content = EmbedContent::new(String::new()).description(text);

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}

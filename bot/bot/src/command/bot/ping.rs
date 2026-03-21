use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use fluent_templates::fluent_bundle::FluentValue;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::{Loader, USABLE_LOCALES};
use std::borrow::Cow;
use std::collections::HashMap;
use tracing::warn;

#[slash_command(
	name = "ping", desc = "Get the ping of the bot (and the shard id).",
	command_type = SubCommand(parent = "bot"),
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
)]
async fn ping_command(self_: PingCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);

	let lang_id = cx.lang_id().await;

	let shard_runner_info = cx.ctx.runner_info.read();

	let shard_id = cx.ctx.shard_id;

	let (latency, stage) = {
		let latency = match shard_runner_info.latency {
			Some(latency) => {
				format!("{:.2}ms", latency.as_millis())
			},
			None => {
				warn!("Latency information not available for shard {}", shard_id);
				"?,??ms".to_string()
			},
		};

		let stage = shard_runner_info.stage.to_string();
		drop(shard_runner_info);
		(latency, stage)
	};

	let mut args = HashMap::new();
	args.insert(
		Cow::Borrowed("shard"),
		FluentValue::from(shard_id.to_string()),
	);
	args.insert(Cow::Borrowed("latency"), FluentValue::from(latency));
	args.insert(Cow::Borrowed("status"), FluentValue::from(stage));

	let title = USABLE_LOCALES.lookup(&lang_id, "bot_ping-title");
	let description = USABLE_LOCALES.lookup_with_args(&lang_id, "bot_ping-desc", &args);

	let embed_content = EmbedContent::new(title).description(description);

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}

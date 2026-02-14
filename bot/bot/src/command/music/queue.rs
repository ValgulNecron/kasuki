//! Documentation for QueueCommand and associated functionality
use crate::command::command::CommandRun;
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use anyhow::anyhow;
use fluent_templates::fluent_bundle::FluentValue;
use futures::future;
use futures::StreamExt;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::{get_language_identifier, Loader, USABLE_LOCALES};
use std::borrow::Cow;
use std::collections::HashMap;

#[slash_command(
	name = "queue", desc = "Show the current queue.",
	command_type = SubCommand(parent = "music"),
	contexts = [Guild],
	install_contexts = [Guild],
)]
async fn queue_command(self_: QueueCommand) -> Result<EmbedsContents<'_>> {
	self_.defer().await?;
	let ctx = self_.get_ctx();
	let bot_data = ctx.data::<BotData>().clone();

	// Retrieve the guild ID from the command interaction
	let guild_id_str = match self_.command_interaction.guild_id {
		Some(id) => id.to_string(),
		None => String::from("0"),
	};
	let db_connection = bot_data.db_connection.clone();

	// Load the localized strings
	let lang_id = get_language_identifier(guild_id_str, db_connection).await;

	let command_interaction = self_.get_command_interaction();

	let guild_id = command_interaction.guild_id.ok_or(anyhow!("no guild id"))?;
	let lava_client = bot_data.lavalink.clone();
	let lava_client = lava_client.read().await.clone();
	if lava_client.is_none() {
		return Err(anyhow::anyhow!("Lavalink is disabled"));
	}
	let lava_client = lava_client.unwrap();
	let Some(player) =
		lava_client.get_player_context(lavalink_rs::model::GuildId::from(guild_id.get()))
	else {
		let embed_content = EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "music_queue-title"))
			.description(USABLE_LOCALES.lookup(&lang_id, "music_queue-error_no_voice"));

		let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

		return Ok(embed_contents);
	};
	let queue = player.get_queue();
	let player_data = player.get_player().await?;

	let max = queue.get_count().await?.min(9);

	let requested_by = USABLE_LOCALES.lookup(&lang_id, "music_queue-requested_by");
	let queue_message = queue
		.enumerate()
		.take_while(|(idx, _)| future::ready(*idx < max))
		.map(|(idx, x)| {
			if let Some(uri) = &x.track.info.uri {
				format!(
					"{} -> [{} - {}](<{}>) | {} <@!{}>",
					idx + 1,
					x.track.info.author,
					x.track.info.title,
					uri,
					requested_by,
					x.track.user_data.unwrap()["requester_id"]
				)
			} else {
				format!(
					"{} -> {} - {} | {} <@!{}",
					idx + 1,
					x.track.info.author,
					x.track.info.title,
					requested_by,
					x.track.user_data.unwrap()["requester_id"]
				)
			}
		})
		.collect::<Vec<_>>()
		.await
		.join("\n");

	let now_playing_message = if let Some(track) = player_data.track {
		let time_s = player_data.state.position / 1000 % 60;
		let time_m = player_data.state.position / 1000 / 60;
		let time = format!("{:02}:{:02}", time_m, time_s);

		let mut args: HashMap<Cow<'static, str>, FluentValue> = HashMap::new();
		args.insert(Cow::Borrowed("var0"), FluentValue::from(track.info.author.clone()));
		args.insert(Cow::Borrowed("var1"), FluentValue::from(track.info.title.clone()));
		args.insert(Cow::Borrowed("var2"), FluentValue::from(track.info.uri.clone().unwrap_or_default()));
		args.insert(Cow::Borrowed("var3"), FluentValue::from(time));
		args.insert(Cow::Borrowed("var4"), FluentValue::from(format!("<@!{}>", track.user_data.unwrap()["requester_id"])));

		USABLE_LOCALES.lookup_with_args(&lang_id, "music_queue-now_playing", &args)
	} else {
		USABLE_LOCALES.lookup(&lang_id, "music_queue-nothing_playing")
	};

	let embed_content = EmbedContent::new(now_playing_message).description(queue_message);

	let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

	Ok(embed_contents)
}

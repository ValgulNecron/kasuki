//! The `PauseCommand` struct represents a command to pause the currently playing
//! track in a voice channel. This command is invoked through a user interaction
//! (slash command) in a Discord server.
//!
//! # Fields
//! * `ctx` - An instance of `SerenityContext` providing access to Discord's API.
//! * `command_interaction` - The CommandInteraction object containing information about the interaction.
use crate::command::command::CommandRun;
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use anyhow::anyhow;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::{get_language_identifier, Loader, USABLE_LOCALES};

#[slash_command(
	name = "pause", desc = "Pause the current song.",
	command_type = SubCommand(parent = "music"),
	contexts = [Guild],
	install_contexts = [Guild],
)]
async fn pause_command(self_: PauseCommand) -> Result<EmbedsContents<'_>> {
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
		let embed_content = EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "music_pause-title"))
			.description(USABLE_LOCALES.lookup(&lang_id, "music_pause-error_no_voice"));

		let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);
		return Ok(embed_contents);
	};
	player.set_pause(true).await?;

	let embed_content = EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "music_pause-title"))
		.description(USABLE_LOCALES.lookup(&lang_id, "music_pause-success"));

	let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);
	Ok(embed_contents)
}

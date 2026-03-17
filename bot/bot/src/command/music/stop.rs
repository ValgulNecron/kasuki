//! The `StopCommand` struct represents a command that stops music playback in a voice channel.
//! It implements the `Command` trait representing a bot command interface.
//!
//! # Fields
//! - `ctx`: The context of the bot, used to access shared data, including bot state and framework.
//! - `command_interaction`: Represents the slash command interaction that triggered this command.
//!
//! # Trait Implementations
//!
//! ## `Command`
//!
//! ### Methods
//!
//! - `get_ctx`
//!   Retrieves the bot context (`SerenityContext`) associated with this command execution.
//!   This context provides access to bot-related data needed for command logic.
//!
//! - `get_command_interaction`
//!   Retrieves the slash command interaction (`CommandInteraction`) that triggered this command.
//!   This interaction holds the details of the interaction, including user input and guild details.
//!
//! - `get_contents`
//!   Asynchronously prepares and returns the response content for the stop command execution. The response
//!   is returned as a vector of `EmbedContent` items which can be sent as bot messages. The logic performs
//!   the following steps:
//!
//!   1. **Retrieving Guild ID**:
//!      Extracts the `guild_id` from the command interaction. If no guild ID is available, an error is raised.
//!
//!   2. **Localization Setup**:
//!      Calls `load_localization_stop` to fetch localized strings for constructing the bot's response,
//!      depending on the guild-specific database.
//!
//!   3. **Initializing Lavalink Player**:
//!      It accesses the Lavalink client instance from shared bot data to control the music playback.
//!      - If Lavalink is disabled or uninitialized, it returns an appropriate message.
//!      - If no active audio player (`player`) is found for the guild, it also provides an error response.
//!
//!   4. **Stopping Music Playback**:
//!      Stops the currently playing track (if
use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use anyhow::anyhow;
use fluent_templates::fluent_bundle::FluentValue;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::{Loader, USABLE_LOCALES};
use std::borrow::Cow;
use std::collections::HashMap;

#[slash_command(
	name = "stop", desc = "Stop the current song.",
	command_type = SubCommand(parent = "music"),
	contexts = [Guild],
	install_contexts = [Guild],
)]
async fn stop_command(self_: StopCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);

	// Load the localized strings
	let lang_id = cx.lang_id().await;

	let guild_id = cx.command_interaction.guild_id.ok_or(anyhow!("no guild id"))?;

	let lava_client = cx.bot_data.lavalink.read().await.clone();
	if lava_client.is_none() {
		return Err(anyhow::anyhow!("Lavalink is disabled"));
	}
	let lava_client = lava_client.unwrap();
	let Some(player) =
		lava_client.get_player_context(lavalink_rs::model::GuildId::from(guild_id.get()))
	else {
		let embed_content = EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "music_stop-title"))
			.description(USABLE_LOCALES.lookup(&lang_id, "music_stop-error_no_voice"));

		let embed_contents = EmbedsContents::new(vec![embed_content]);

		return Ok(embed_contents);
	};
	let mut embed_content = EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "music_stop-title"));

	let now_playing = player.get_player().await?.track;

	if let Some(np) = now_playing {
		player.stop_now().await?;
		let mut args: HashMap<Cow<'static, str>, FluentValue> = HashMap::new();
		args.insert(
			Cow::Borrowed("var0"),
			FluentValue::from(np.info.title.clone()),
		);
		embed_content = embed_content.description(USABLE_LOCALES.lookup_with_args(
			&lang_id,
			"music_stop-success",
			&args,
		));
	} else {
		embed_content = embed_content
			.description(USABLE_LOCALES.lookup(&lang_id, "music_stop-nothing_to_stop"));
	}

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}

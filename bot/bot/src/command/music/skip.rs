//! The `SkipCommand` struct represents the functionality of a music skip command in a Discord bot.
//!
//! It implements the `Command` trait to handle retrieving the context and
//! command interaction, deferring responses, and processing the skip logic.
//!
//! The `SkipCommand` retrieves the current playing track, attempts to skip it if possible,
//! and sends appropriate feedback to the user.
//!
//! # Fields
//! - `ctx`: Represents the context in which the command is executed.
//! - `command_interaction`: Represents the interaction associated with this specific command.
//!
//! # Implementations
//! ## `get_ctx`
//! Retrieves the execution context (`SerenityContext`) passed to the command.
//!
//! ## `get_command_interaction`
//! Retrieves the command interaction (`CommandInteraction`) associated with the given command.
//!
//! ## `get_contents`
//! Handles the main logic behind the skip functionality and provides user feedback.
//!
//! This method:
//! 1. Retrieves the guild ID and loads localized strings for the skip command feedback (title, errors, messages).
//! 2. Verifies if the Lavalink client (music client) exists and retrieves the player context for the associated guild.
//! 3. Checks if there is a currently playing track:
//!    - If a track is playing, it skips the track and informs the user of the skipped track's title.
//!    - If no track is playing, it sends feedback indicating there is nothing to skip.
//!
//! ### Errors
//! - Returns an error if:
//!     - Lavalink client is not available or disabled.
//!     - The guild ID cannot be found.
//!     - Playback operations fail or the player context cannot be retrieved.
//!
//! ### Returns
//! - A vector of `EmbedContent` objects containing the message(s) to be sent as bot feedback.
//!
//! # Example Usage
//! When a user sends a `/skip` command:
//! - If a song is currently playing, the bot skips the track and sends a success message.
//! - If no song is playing or the Lavalink client is unavailable, the bot sends an error message.
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
	name = "skip", desc = "Skip the current song.",
	command_type = SubCommand(parent = "music"),
	contexts = [Guild],
	install_contexts = [Guild],
)]
async fn skip_command(self_: SkipCommand) -> Result<EmbedsContents<'_>> {
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
		let embed_content = EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "music_skip-title"))
			.description(USABLE_LOCALES.lookup(&lang_id, "music_skip-error_no_voice"));

		let embed_contents = EmbedsContents::new(vec![embed_content]);

		return Ok(embed_contents);
	};
	let mut embed_content = EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "music_skip-title"));

	let now_playing = player.get_player().await?.track;

	if let Some(np) = now_playing {
		player.skip()?;
		let mut args: HashMap<Cow<'static, str>, FluentValue> = HashMap::new();
		args.insert(
			Cow::Borrowed("var0"),
			FluentValue::from(np.info.title.clone()),
		);
		embed_content = embed_content.description(USABLE_LOCALES.lookup_with_args(
			&lang_id,
			"music_skip-success",
			&args,
		));
	} else {
		embed_content = embed_content
			.description(USABLE_LOCALES.lookup(&lang_id, "music_skip-nothing_to_skip"));
	}

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}

//! The `SeekCommand` struct and its implementation are used to handle the "seek" interaction
//! in a bot, allowing users to seek or jump to a specific point within the currently playing track.
//!
//! # Struct Fields
//! - `ctx: SerenityContext`
//!   - Provides the context of the interaction, including the bot's state.
//! - `command_interaction: CommandInteraction`
//!   - Represents the user interaction with the command.
//!
//! # Traits Implemented
//! ## `Command`
//! The `SeekCommand` struct implements the `Command` trait, enabling it to fulfill the behavior
//! required for handling seek operations in music playback. Below are the implemented methods:
//!
//! ### `get_ctx(&self) -> &SerenityContext`
//! Returns the bot context associated with the current command.
//!
//! ### `get_command_interaction(&self) -> &CommandInteraction`
//! Returns the interaction that triggered the command.
//!
//! ### `get_contents(&self) -> anyhow::Result<Vec<EmbedContent<'_, '_>>>`
//! Handles the main logic for the seek command, including:
//! - Retrieving and verifying the guild ID to which the command belongs.
//! - Loading localized messages for responses based on the guild.
//! - Checking the availability of the Lavalink music client.
//! - Attempting to get the current music player for the guild.
//! - Seeking to the specified point in the currently playing track.
//! - Building response embeds with appropriate output messages (e.g., success or error).
//!
//! #### Behavior of `get_contents`
//! 1. The function defers the reply to the interaction for processing.
//! 2. Loads localized strings for the seek command using the guild ID.
//! 3. Fetches a reference to the Lavalink client for music playback management:
//!    - If Lavalink is disabled or uninitialized, returns an error message in an embed.
//! 4. Extracts the "time" option provided by the user (via subcommand input) to determine
//!    the seek position in seconds.
//! 5. Checks if a track is currently playing in the guild:
//!    - If a track is playing, seeks the track to the specified position.
//!    - If no track is playing, returns an appropriate error message as a response.
//! 6. Constructs an embed with the success or error result.
//! 7. Returns the generated embed content.
//!
//! #### Errors
//! The `get_contents` method produces an error if:
//! - Lavalink is disabled or unavailable.
//! - The guild has no active voice connection or player.
//! - The "time" option is missing, or seeking to the given position fails.
//!
//! ```
//! // Example usage:
//! let seek_command = SeekCommand {
//!     ctx: serenity_context,
//!     command_interaction,
//! };
//! seek_command.get_contents().await?;
//! ```
//!
//! # Dependencies
//! - Uses the Lavalink library to handle music playback.
//! - Depends on localized messages loaded via `load_localization_seek`.
//! - Relies on extracted subcommand options for the seek time.
//!
//! # Notes
//! - This command only functions within a guild context where Lavalink is properly configured.
//! - If no track is currently playing in the guild, the command will notify the user accordingly.
use crate::command::command::CommandRun;
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand::get_option_map_number_subcommand;
use anyhow::anyhow;
use fluent_templates::fluent_bundle::FluentValue;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::{get_language_identifier, Loader, USABLE_LOCALES};
use std::borrow::Cow;
use std::collections::HashMap;
use std::time::Duration;

#[slash_command(
	name = "seek", desc = "Seek to a position in the current song.",
	command_type = SubCommand(parent = "music"),
	contexts = [Guild],
	install_contexts = [Guild],
	args = [(name = "time", desc = "Time to seek to in seconds.", arg_type = Integer, required = true, autocomplete = false)],
)]
async fn seek_command(self_: SeekCommand) -> Result<EmbedsContents<'_>> {
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
		let embed_content =
			EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "music_seek-title")).description(USABLE_LOCALES.lookup(&lang_id, "music_seek-error_no_voice"));

		let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

		return Ok(embed_contents);
	};

	let map = get_option_map_number_subcommand(command_interaction);

	let time = map.get(&String::from("time")).cloned().unwrap_or_default() as u64;

	let now_playing = player.get_player().await?.track;

	let mut embed_content = EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "music_seek-title"));

	if let Some(_) = now_playing {
		player.set_position(Duration::from_secs(time)).await?;
		let mut args: HashMap<Cow<'static, str>, FluentValue> = HashMap::new();
		args.insert(Cow::Borrowed("var0"), FluentValue::from(time.to_string()));
		embed_content =
			embed_content.description(USABLE_LOCALES.lookup_with_args(&lang_id, "music_seek-success", &args));
	} else {
		embed_content = embed_content.description(USABLE_LOCALES.lookup(&lang_id, "music_seek-nothing_playing"));
	}

	let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

	Ok(embed_contents)
}

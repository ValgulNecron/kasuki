//! The `SwapCommand` struct and its implementation define the functionality to swap two tracks
//! in a music player's queue in response to a command interaction.
//!
//! # Fields
//! - `ctx`: The context from Serenity, used to interact with the Discord bot.
//! - `command_interaction`: An interaction object representing the executed command.
//!
//! # Implementation
//! The `SwapCommand` struct implements the `Command` trait, which provides the following functionality:
//!
//! ## Methods
//!
//! ### `get_ctx`
//! Returns a reference to the Serenity context.
//!
//! ```rust
//! fn get_ctx(&self) -> &SerenityContext
//! ```
//!
//! ### `get_command_interaction`
//! Returns a reference to the command interaction object.
//!
//! ```rust
//! fn get_command_interaction(&self) -> &CommandInteraction
//! ```
//!
//! ### `get_contents`
//! Handles the core logic for processing the "swap" command. It performs the following:
//! - Retrieves the necessary bot data from the context.
//! - Defers the interaction response to indicate processing time.
//! - Loads localization strings for the swap command based on the guild ID.
//! - Validates if the Lavalink music player client is enabled.
//! - Checks if the indices (`index1`, `index2`) provided by the user are valid for the queue.
//! - Swaps the tracks at the specified indices in the player's music queue.
//! - Builds an embed response to indicate success or errors (e.g., invalid indices, Lavalink not running).
//!
//! #### Parameters
//! None directly (uses `self`).
//!
//! #### Returns
//! An `anyhow::Result` containing `EmbedsContents` on success, or an error if something went wrong.
//!
//! #### Example Workflow
//! 1. Determines if the command interaction has a valid guild ID.
//! 2. Loads localized strings for success or error messages.
//! 3. Retrieves the Lavalink player's queue for the current guild.
//! 4. Validates and processes the `index1` and `index2` options from the command interaction.
//! 5. Swaps the tracks in the queue and constructs an embed message with the result.
//!
//! #### Important Details
//! - If no valid guild ID is found, an error is returned.
//! - If the indices are greater than the queue length or the same, appropriate error messages are shown using embed content.
//! - If Lavalink is disabled or unavailable, the command fails.
//!
//! ```rust
//! async fn get_contents(&self) -> anyhow::Result<EmbedsContents>
//! ```
//!
//! # Dependencies
//! This implementation uses several modules and crates:
//! - `serenity::all`: Handles Discord API interactions.
//! - Local modules such as:
//!   - `command`: Defines command-related structs and enums (e.g., `Command`, `EmbedContent`).
//!   - `event_handler`: Provides bot data configurations.
//!   - `helper`: Includes utility functions (e.g., retrieving command options).
//! - `anyhow`: For error handling.
//! - `lavalink_rs`: Manages interactions with the Lavalink music streaming server.
//!
//! # Errors
//! The `get_contents` method may return errors in the following scenarios:
//! - Missing guild ID (`no guild id`).
//! - Lavalink is not enabled or available.
//! - Invalid or missing indices to swap in the queue.
//!
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

#[slash_command(
	name = "swap", desc = "Swap two songs in the queue.",
	command_type = SubCommand(parent = "music"),
	contexts = [Guild],
	install_contexts = [Guild],
	args = [
		(name = "index1", desc = "Index of the first song.", arg_type = Integer, required = true, autocomplete = false),
		(name = "index2", desc = "Index of the second song.", arg_type = Integer, required = true, autocomplete = false)
	],
)]
async fn swap_command(self_: SwapCommand) -> Result<EmbedsContents<'_>> {
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
			EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "music_swap-title")).description(USABLE_LOCALES.lookup(&lang_id, "music_swap-error_no_voice"));

		let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

		return Ok(embed_contents);
	};

	let map = get_option_map_number_subcommand(command_interaction);

	let index1 = map
		.get(&String::from("index1"))
		.cloned()
		.unwrap_or_default() as usize;

	let index2 = map
		.get(&String::from("index2"))
		.cloned()
		.unwrap_or_default() as usize;

	let mut embed_content = EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "music_swap-title"));

	let queue = player.get_queue();
	let queue_len = queue.get_count().await?;

	if index1 > queue_len || index2 > queue_len {
		let mut args: HashMap<Cow<'static, str>, FluentValue> = HashMap::new();
		args.insert(Cow::Borrowed("var0"), FluentValue::from(queue_len.to_string()));
		embed_content = embed_content.description(
			USABLE_LOCALES.lookup_with_args(&lang_id, "music_swap-error_max_index", &args),
		);
	} else if index1 == index2 {
		embed_content = embed_content.description(USABLE_LOCALES.lookup(&lang_id, "music_swap-error_same_index"));
	} else {
		let track1 = queue.get_track(index1 - 1).await?.unwrap();
		let track2 = queue.get_track(index1 - 2).await?.unwrap();

		queue.swap(index1 - 1, track2)?;
		queue.swap(index2 - 1, track1)?;

		embed_content = embed_content.description(USABLE_LOCALES.lookup(&lang_id, "music_swap-success"));
	}

	let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

	Ok(embed_contents)
}

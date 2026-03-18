//! `ResumeCommand` is a concrete implementation of the `Command` trait, which resumes
//! the playback of music in a voice channel when invoked through a bot command.
//!
//! # Fields
//! - `ctx`:
//!   The `SerenityContext` object, providing access to the bot's core context, state, and utilities.
//! - `command_interaction`:
//!   Represents the interaction instance received from the Discord API when the command is triggered.
//!
//! # Trait Implementations
//!
//! ## `Command` Trait
//!
//! The `ResumeCommand` struct implements the following trait functions as part of the `Command` trait:
//!
//! ### `get_ctx(&self) -> &SerenityContext`
//!
//! Provides a reference to the `SerenityContext` associated with the command.
//!
//! **Returns:**
//! A reference to the context for the bot.
//!
//! ### `get_command_interaction(&self) -> &CommandInteraction`
//!
//! Provides a reference to the `CommandInteraction` instance that represents the
//! command invoked by the user.
//!
//! **Returns:**
//! A reference to the command interaction.
//!
//! ### `async fn get_contents(&self) -> anyhow::Result<Vec<EmbedContent<'_, '_>>>`
//!
//! Handles the core logic of the "resume" music command. It performs the following operations:
//!
//! 1. Retrieves the `BotData` shared context to access configurations, Lavalink state, etc.
//! 2. Defer the initial response for the command interaction to manage latency expectations.
//! 3. Retrieves the localized messages specific to the guild where the command was called.
//! 4. Verifies if the command was called in a valid guild voice channel.
//! 5. Checks if the Lavalink client is available and retrieves the player context for the guild.
//! 6. Resumes music playback by un-pausing the Lavalink player for the corresponding guild.
//! 7. Sends an appropriate response back to the Discord user based on the outcome:
//!    - If successful, an embed with a success message is returned.
//!    - If an error occurs (e.g., no active voice connection), an error message is returned.
//!
//! **Returns:**
//! - `Ok(Vec<EmbedContent<'_, '_>>)` on success:
//!   A vector containing embed content to be sent back as a response.
//! - `Err(anyhow::Error)` on failure:
//!   An error if the Lavalink client is disabled, the bot has no active voice player,
//!   or if any other issue arises during execution.
//!
//! **Errors:**
//! - If the bot is invoked in a context where no guild ID is provided.
//! - If the Lavalink client is unavailable or disabled.
//! - If no active voice player exists in the guild.
//!
//! # Dependencies
//! - `SerenityContext` for interacting with the Discord API and bot state.
//! - `CommandInteraction` to process commands from users.
//! - `Lavalink` for managing music playback.
//! - Utility functions like `load_localization_resume` to fetch localized strings.
//!
//! # Example
//! ```ignore
//! let resume_command = ResumeCommand {
//!     ctx: serenity_context,
//!     command_interaction: command_interaction_instance,
//! };
//!
//! match resume_command.get_contents().await {
//!     Ok(embed_contents) => {
//!         for embed in embed_contents {
//!             println!("Embed title: {}", embed.title);
//!         }
//!     }
//!     Err(error) => {
//!         println!("An error occurred: {}", error);
//!     }
//! }
//! ```
use crate::command::context::CommandContext;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use anyhow::anyhow;
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::{Loader, USABLE_LOCALES};

#[slash_command(
	name = "resume", desc = "Resume the current song.",
	command_type = SubCommand(parent = "music"),
	contexts = [Guild],
	install_contexts = [Guild],
)]
async fn resume_command(self_: ResumeCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);

	// Load the localized strings
	let lang_id = cx.lang_id().await;

	let guild_id = cx
		.command_interaction
		.guild_id
		.ok_or(anyhow!("no guild id"))?;
	let lava_client = cx.bot_data.lavalink.read().await.clone();
	if lava_client.is_none() {
		return Err(anyhow::anyhow!("Lavalink is disabled"));
	}
	let lava_client = lava_client.unwrap();
	let Some(player) =
		lava_client.get_player_context(lavalink_rs::model::GuildId::from(guild_id.get()))
	else {
		let embed_content =
			EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "music_resume-title"))
				.description(USABLE_LOCALES.lookup(&lang_id, "music_resume-error_no_voice"));

		let embed_contents = EmbedsContents::new(vec![embed_content]);

		return Ok(embed_contents);
	};

	player.set_pause(false).await?;

	let embed_content = EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "music_resume-title"))
		.description(USABLE_LOCALES.lookup(&lang_id, "music_resume-success"));

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}

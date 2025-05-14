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
use crate::command::command::{Command, CommandRun, EmbedContent, EmbedType};
use crate::event_handler::BotData;
use crate::structure::message::music::resume::load_localization_resume;
use anyhow::anyhow;
use serenity::all::{CommandInteraction, Context as SerenityContext};

///
/// The `ResumeCommand` struct is responsible for handling the "resume" command functionality
/// within a Discord bot application powered by the `serenity` library. This struct encapsulates
/// the context of the bot and the interaction data associated with invoking the command.
///
/// # Fields
///
/// * `ctx` (`SerenityContext`)  
///   Represents the context in which the bot operates, provided by the `serenity` library.
///   This includes data needed to interact with Discord's API and other runtime information.
///
/// * `command_interaction` (`CommandInteraction`)  
///   Encapsulates the interaction data when a user invokes the "resume" command. This includes
///   metadata about the command, the user who issued it, and additional parameters.
///
/// # Example
///
/// ```rust
/// // Assuming you have the relevant context and CommandInteraction ready:
/// let resume_command = ResumeCommand {
///     ctx: serenity_context,
///     command_interaction: command_interaction_data,
/// };
///
/// // Implement and invoke the logic to handle the resume command.
/// resume_command.execute();
/// ```
pub struct ResumeCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for ResumeCommand {
	/// Retrieves a reference to the `SerenityContext` instance associated with the current object.
	///
	/// # Returns
	/// A reference to the `SerenityContext` (`&SerenityContext`) held by the object.
	///
	/// # Example
	/// ```rust
	/// let context = some_object.get_ctx();
	/// // Use `context` for further operations within the Serenity framework.
	/// ```
	///
	/// This function provides access to the context, which is generally used in interacting
	/// with the Serenity Discord library for handling events, data, and bot actions.
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` associated with the instance.
	///
	/// # Returns
	///
	/// A reference to the `CommandInteraction` object stored in the instance.
	///
	/// # Examples
	///
	/// ```rust
	/// // Assuming `self` is an instance of a struct that includes `command_interaction`
	/// let command_interaction = self.get_command_interaction();
	/// ```
	///
	/// This can be used to access and interact with the `CommandInteraction` for
	/// further processing or data retrieval.
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Asynchronously retrieves and processes the contents required for resuming audio playback in a Discord guild.
	///
	/// # Returns
	/// Returns a `Vec` of `EmbedContent` encapsulating localized strings and status messages
	/// to be used as responses for the user interaction.
	///
	/// # Errors
	/// - Returns an error if:
	///     - Retrieving the Lavalink client fails.
	///     - Lavalink functionality is disabled.
	///     - The bot is not connected to a voice channel in the guild.
	///     - The guild ID is not available from the command interaction.
	///     - Localization strings cannot be loaded for the guild.
	///
	/// # Process
	/// 1. Obtains the context of the bot (`ctx`) and extracts shared data (`bot_data`).
	/// 2. Attempts to defer the response to acknowledge the interaction with Discord.
	/// 3. Retrieves the guild ID from the command interaction; if unavailable, assigns a default value of `"0"`.
	/// 4. Fetches localized strings for resuming playback from the database associated with the bot configuration.
	/// 5. Validates if Lavalink audio services are properly initialized and accessible.
	/// 6. Checks for the presence of a player context in Lavalink for the specified guild:
	///     - If unavailable, creates an error response indicating the bot is not in a voice channel.
	/// 7. Resumes playback by unpausing the Lavalink player for the guild.
	/// 8. Constructs a success response using localized strings and prepares it for return.
	///
	/// # Notes
	/// - This function relies on the `lavalink_rs` library for managing audio playback in Discord.
	/// - Localization strings are fetched based on the guild's ID and the bot's configuration.
	/// - The response content is structured using the `EmbedContent` and `EmbedType` types.
	///
	/// # Dependencies
	/// - `anyhow::Result`: For error handling.
	/// - `EmbedContent` and `EmbedType`: For structuring interaction responses.
	/// - `load_localization_resume`: For loading localized strings based on guild-specific context.
	/// - `lavalink_rs`: For interacting and managing audio playback.
	///
	/// # Example
	/// ```ignore
	/// let result = my_interaction_handler.get_contents().await;
	/// match result {
	///     Ok(embeds) => {
	///         // Send these embeds as a response to the user's command
	///     }
	///     Err(error) => {
	///         // Handle the error (e.g., log it or send an error message back to the user)
	///     }
	/// }
	/// ```
	async fn get_contents(&self) -> anyhow::Result<Vec<EmbedContent<'_, '_>>> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		self.defer().await?;

		// Retrieve the guild ID from the command interaction
		let guild_id_str = match self.command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};

		// Load the localized strings
		let resume_localised =
			load_localization_resume(guild_id_str, bot_data.config.db.clone()).await?;

		let command_interaction = self.get_command_interaction();

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
			let embed_content = EmbedContent::new(resume_localised.title)
				.description(resume_localised.error_no_voice)
				.command_type(EmbedType::Followup);
			return Ok(vec![embed_content]);
		};

		player.set_pause(false).await?;

		let embed_content = EmbedContent::new(resume_localised.title)
			.description(resume_localised.success)
			.command_type(EmbedType::Followup);

		Ok(vec![embed_content])
	}
}

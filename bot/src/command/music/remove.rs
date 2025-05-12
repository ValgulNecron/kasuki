//! This module defines the `RemoveCommand`, a structure and implementation 
//! used to handle the "remove" functionality within a bot command interaction. 
//! The "remove" command allows users to remove a track from the music queue.
use crate::command::command_trait::{Command, CommandRun, EmbedContent, EmbedType};
use crate::event_handler::BotData;
use crate::helper::get_option::subcommand::get_option_map_number_subcommand;
use crate::structure::message::music::remove::load_localization_remove;
use anyhow::anyhow;
use serenity::all::{CommandInteraction, Context as SerenityContext};

/// The `RemoveCommand` struct represents a command to handle the removal
/// of an entity or item in a Discord bot using the Serenity library.
///
/// This struct encapsulates the context and the specific command interaction
/// details required to execute the "Remove" command.
///
/// # Fields
///
/// * `ctx` - The shared context (`SerenityContext`) instance that provides access to
///           interaction with the Discord API, as well as other components of the bot's state.
///
/// * `command_interaction` - The `CommandInteraction` object containing details about
///                           the command issued by the user, including its arguments
///                           and metadata.
///
/// # Usage
///
/// The `RemoveCommand` struct is typically used as part of a command execution handler,
/// where it provides necessary context and data for processing a removal-related operation.
///
/// This would require you to:
/// 1. Extract relevant information from `command_interaction`.
/// 2. Leverage the `ctx` field to perform any necessary API calls to Discord or other operations.
///
/// Both fields are expected to be passed when this struct is instantiated.
pub struct RemoveCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for RemoveCommand {
	/// Retrieves a reference to the `SerenityContext`.
	///
	/// This method provides access to the `SerenityContext` associated with the current instance,
	/// allowing the caller to work with Discord-related context information within the Serenity framework.
	///
	/// # Returns
	/// * `&SerenityContext` - A reference to the `SerenityContext` contained in the current instance.
	///
	/// # Example
	/// ```rust
	/// let context = instance.get_ctx();
	/// ```
	///
	/// This is typically used for interacting with the Discord API or handling events in a bot.
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` associated with the current instance.
	///
	/// # Returns
	/// - A reference to the `CommandInteraction` instance (`&CommandInteraction`) stored within `self`.
	///
	/// # Examples
	/// ```rust
	/// let command_interaction = instance.get_command_interaction();
	/// // Use the returned `CommandInteraction` reference.
	/// ```
	///
	/// # Note
	/// This function provides read-only access to the `command_interaction` field and does not modify the state of `self`.
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Asynchronously retrieves the contents for a bot interaction.
	///
	/// This function handles the interaction logic within the context of a bot command and
	/// provides relevant responses based on the Lavalink player's state and queue operations. 
	/// It defers the response initially and processes the necessary subcommand details.
	///
	/// # Returns
	/// A result containing a vector of `EmbedContent` elements on success, or an `anyhow::Error`
	/// on failure.
	///
	/// # Errors
	/// The function may return errors in the following scenarios:
	/// - Interaction lacks a guild ID (`"no guild id"` error).
	/// - Lavalink client is disabled or unavailable.
	/// - Specified music queue index for removal is invalid.
	///
	/// # Steps in the Process
	/// 1. Retrieves the command interaction's guild ID. If absent, assumes a default ID.
	/// 2. Loads localized strings for the "remove" functionality from the database.
	/// 3. Fetches the Lavalink client. Returns an error if Lavalink is not enabled.
	/// 4. Retrieves the Lavalink player's context for the current guild. If the player does not exist,
	///    prepares a follow-up embed with an error message for no active voice session.
	/// 5. Extracts the "index" option from the interaction's subcommand (defaulting to `0` if unspecified)
	///    and attempts to remove the track at the specified index from the player's queue.
	/// 6. Constructs and returns a success embed response if the track is removed successfully.
	///
	/// # Parameters
	/// - `&self`: A reference to the current object/context (`self`) containing
	///   the interaction and additional context for the bot.
	///
	/// # Example
	/// ```rust
	/// async fn process_interaction() -> anyhow::Result<()> {
	///     let interaction = MyInteraction::new(); // Your interaction struct instance
	///     let response = interaction.get_contents().await?;
	///     for embed in response {
	///         println!("Embed: {}", embed.description); // Example of utilizing the response
	///     }
	///     Ok(())
	/// }
	/// ```
	///
	/// # Relevant Types
	/// - `EmbedContent<'_, '_>`: A structure representing embed responses with customizable title,
	///    description, and type.
	/// - `BotData`: The global bot data that includes the Lavalink client and database configuration.
	/// - `EmbedType`: Enum representing the type of embed (e.g., Followup, Response).
	///
	/// This function is typically called in the context of handling a "remove" subcommand
	/// that modifies the music playback queue in a guild's voice session.
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
		let remove_localised =
			load_localization_remove(guild_id_str, bot_data.config.db.clone()).await?;

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
			let embed_content = EmbedContent::new(remove_localised.title)
				.description(remove_localised.error_no_voice)
				.command_type(EmbedType::Followup);
			return Ok(vec![embed_content]);
		};

		let map = get_option_map_number_subcommand(command_interaction);

		let index = map.get(&String::from("index")).cloned().unwrap_or_default() as usize;

		player.get_queue().remove(index)?;

		let embed_content = EmbedContent::new(remove_localised.title)
			.description(remove_localised.success)
			.command_type(EmbedType::Followup);
		
		Ok(vec![embed_content])
	}
}

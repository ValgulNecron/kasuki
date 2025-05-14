//! The `JoinCommand` struct is responsible for handling the join command interaction
//! from a Discord guild. When the join command is invoked, the bot attempts to join
//! the voice channel of the user who issued the command.
//!
//! # Attributes
//! - `ctx`: Contains the Serenity context, which provides access to the bot's internal state, caches, and configurations.
//! - `command_interaction`: Represents the interaction data triggered by a user's slash or text command.
use crate::command::command::{Command, CommandRun, EmbedContent, EmbedType};
use crate::event_handler::BotData;
use crate::structure::message::music::join::load_localization_join;
use anyhow::{Result, anyhow};
use lavalink_rs::model::ChannelId;
use serenity::all::{CommandInteraction, Context as SerenityContext, Context};
use serenity::http::Http;
use serenity::prelude::Mentionable;
use std::sync::Arc;

/// The `JoinCommand` struct represents a command to make a bot join a specific context or perform an action upon invocation.
///
/// This struct encapsulates the required context and interaction details for handling a join command in a Discord bot.
///
/// # Fields
///
/// * `ctx` - The `SerenityContext` which provides the bot with access to Discord API, cache, and other utilities needed to manage the bot's state and operations.
/// * `command_interaction` - The `CommandInteraction` object containing details about the join command interaction, such as the user who invoked it and context around the interaction.
///
/// This struct is used to handle and process user commands related to joining a specific resource or session in the bot workflow.
pub struct JoinCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for JoinCommand {
	/// Retrieves a reference to the `SerenityContext` within the current instance.
	///
	/// # Returns
	/// A reference to the `SerenityContext` associated with this instance.
	///
	/// # Usage
	/// This method provides access to the `SerenityContext`, which contains
	/// information and utilities for interacting with the Discord API.
	///
	/// # Example
	/// ```rust
	/// let context = instance.get_ctx();
	/// // Use the context for Discord API operations
	/// ```
	///
	/// # Notes
	/// - This method borrows the context immutably, so the returned reference
	/// cannot be used to make modifications to the context.
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` associated with the current instance.
	///
	/// # Returns
	/// A reference to the `CommandInteraction` object.
	///
	/// # Example
	/// ```rust
	/// let interaction = instance.get_command_interaction();
	/// // Use the returned `CommandInteraction` reference
	/// ```
	///
	/// This method allows read-only access to the `CommandInteraction` field of the structure.
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Fetches and returns a list of `EmbedContent` asynchronously.
	///
	/// This function performs the following steps:
	/// 1. Retrieves the context (`ctx`) associated with the current state of the bot.
	/// 2. Accesses the bot's shared data (`BotData`) from the context.
	/// 3. Retrieves the command interaction currently being processed.
	/// 4. Defers the interaction to indicate to the user that the bot is working and may take some time to respond.
	/// 5. Joins the necessary information from the context, bot data, and command interaction to produce the desired embed content.
	/// 6. Returns a vector of `EmbedContent` objects encapsulating the relevant information.
	///
	/// ### Returns
	/// - `Ok(Vec<EmbedContent<'_, '_>>)` if the operation is successful, containing the embed content to be processed or displayed.
	/// - `Err(_)` if an error occurs during any of the aforementioned steps.
	///
	/// ### Errors
	/// This function returns an error if:
	/// - Retrieving the context or bot data fails.
	/// - The interaction cannot be deferred properly.
	/// - An error occurs while joining the required data to generate the embed content.
	///
	/// ### Usage
	/// This method is typically called when an interaction requires generation of rich embed responses that aggregate
	/// information from various sources.
	///
	/// ### Example
	/// ```rust
	/// let embed_contents = your_instance.get_contents().await?;
	/// for content in embed_contents {
	///     println!("{:?}", content);
	/// }
	/// ```
	///
	/// ### Dependencies
	/// - The `join` function must support the provided context, bot data, and command interaction to generate embed content.
	/// - This function assumes that `defer` is called successfully to handle interaction delays appropriately.
	///
	/// ### Notes
	/// - Ensure that sharing and cloning bot data is safe and that lifetime requirements align with the provided borrow checks.
	/// - Internal logic might depend on external systems or APIs for joining the context, bot data, and interaction output.
	async fn get_contents(&self) -> Result<Vec<EmbedContent<'_, '_>>> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();

		self.defer().await?;

		let (_, embed_content) = join(ctx, bot_data, command_interaction).await?;

		Ok(embed_content)
	}
}

/// Asynchronously handles the bot's joining of a voice channel in response to a command.
///
/// This function determines the voice channel of the user issuing the command, connects
/// the bot to the identified voice channel within a guild, and sets up the Lavalink player.
/// If any issues are encountered (e.g., the user is not in a voice channel or a connection
/// to the Lavalink service fails), the appropriate feedback is provided.
///
/// # Parameters
/// - `ctx`: Reference to the Discord context, which provides access to the bot's current state.
/// - `bot_data`: Shared reference to the bot's global data, including configurations and Lavalink.
/// - `command_interaction`: Context for the command interaction issued by the user, including
///   guild, channel, and user details.
///
/// # Returns
/// - `Result<(bool, Vec<EmbedContent<'static, 'static>>)>`: On success, returns a tuple containing:
///   - A boolean indicating success or failure of the operation.
///   - A vector of `EmbedContent` objects for user feedback.
///   On error, returns an `anyhow::Error` describing the failure.
///
/// # Behavior
/// - Checks the guild context and user issuing the command to ensure they are in a valid voice channel.
/// - Retrieves localized messages and error handling from the cache or database.
/// - Manages connections to the Lavalink service and sets up the music player for the guild.
/// - Responds with context-specific feedback to the user, such as success or failure embeds.
///
/// # Error Handling
/// - If Lavalink is disabled or unavailable, the function returns an error.
/// - If the user is not in a voice channel, the function provides feedback indicating this issue.
/// - If the guild or voice state information is missing, an error is returned.
///
/// # Examples
/// ```rust
/// let result = join(&ctx, bot_data, &command_interaction).await;
/// match result {
///     Ok((true, embeds)) => {
///         for embed in embeds {
///             println!("Success: {:?}", embed);
///         }
///     }
///     Ok((false, embeds)) => {
///         for embed in embeds {
///             println!("Failure: {:?}", embed);
///         }
///     }
///     Err(err) => {
///         eprintln!("Error: {:?}", err);
///     }
/// }
/// ```
///
/// # Dependencies
/// - `lavalink_rs` for managing voice and music session.
/// - `EmbedContent` and `EmbedType` for constructing Discord embed messages.
/// - `Context`, `BotData`, and `CommandInteraction` for handling interaction and state.
///
/// # Notes
/// - This function assumes that Lavalink is properly configured and running in the bot context.
///   If Lavalink is disabled, the function will exit with an error.
/// - Ensure the voice permissions for the bot are properly granted in the target guild.
pub async fn join(
	ctx: &Context, bot_data: Arc<BotData>, command_interaction: &CommandInteraction,
) -> Result<(bool, Vec<EmbedContent<'static, 'static>>)> {
	// Retrieve the guild ID from the command interaction
	let guild_id_str = match command_interaction.guild_id {
		Some(id) => id.to_string(),
		None => String::from("0"),
	};

	// Load the localized strings
	let join_localised = load_localization_join(guild_id_str, bot_data.config.db.clone()).await?;

	let lava_client = bot_data.lavalink.read().await.clone();
	match lava_client {
		Some(_) => {},
		None => {
			return Err(anyhow::anyhow!("Lavalink is disabled"));
		},
	}
	let lava_client = lava_client.unwrap();
	let manager = bot_data.manager.clone();

	let guild_id = command_interaction.guild_id.ok_or(anyhow!("no guild id"))?;

	// Get the channel information BEFORE creating any futures
	let channel_id = command_interaction.channel_id;
	let author_id = command_interaction.user.id;

	// Extract just the voice channel ID from the guild cache before awaiting anything
	let connect_to = {
		// Extract the voice channel data from the cache
		let guild = guild_id
			.to_guild_cached(&ctx.cache)
			.ok_or(anyhow!("Guild not found"))?;

		let user_channel_id = guild
			.voice_states
			.get(&author_id)
			.and_then(|voice_state| voice_state.channel_id);

		// We only need the channel ID from this scope
		match user_channel_id {
			Some(channel) => channel,
			None => {
				return Ok((
					false,
					vec![
						EmbedContent::new(join_localised.title)
							.description(join_localised.error_no_voice)
							.command_type(EmbedType::Followup),
					],
				));
			},
		}
	};

	// Create the embed content outside the non-Send guild reference scope
	let mut content = EmbedContent::new(join_localised.title).command_type(EmbedType::Followup);

	if lava_client
		.get_player_context(lavalink_rs::model::GuildId::from(guild_id.get()))
		.is_none()
	{
		let handler = manager.join_gateway(guild_id, connect_to).await;

		let (result, return_data) = match handler {
			Ok((connection_info, _)) => {
				lava_client
					.create_player_context_with_data::<(ChannelId, Arc<Http>)>(
						lavalink_rs::model::GuildId::from(guild_id.get()),
						lavalink_rs::model::player::ConnectionInfo {
							endpoint: connection_info.endpoint,
							token: connection_info.token,
							session_id: connection_info.session_id,
						},
						Arc::new((ChannelId(channel_id.get()), ctx.http.clone())),
					)
					.await?;

				content = content.description(
					join_localised
						.success
						.replace("{0}", &connect_to.mention().to_string()),
				);
				(true, content)
			},
			Err(why) => {
				content = content.description(
					join_localised
						.error_joining
						.replace("{0}", &why.to_string()),
				);
				(false, content)
			},
		};
		return Ok((result, vec![return_data]));
	};
	Ok((false, vec![content]))
}

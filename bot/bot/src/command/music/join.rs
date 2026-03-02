//! The `JoinCommand` struct is responsible for handling the join command interaction
//! from a Discord guild. When the join command is invoked, the bot attempts to join
//! the voice channel of the user who issued the command.
//!
//! # Attributes
//! - `ctx`: Contains the Serenity context, which provides access to the bot's internal state, caches, and configurations.
//! - `command_interaction`: Represents the interaction data triggered by a user's slash or text command.
use crate::command::command::CommandRun;
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use anyhow::{anyhow, Result};
use fluent_templates::fluent_bundle::FluentValue;
use kasuki_macros::slash_command;
use lavalink_rs::model::ChannelId;
use serenity::all::{CommandInteraction, Context as SerenityContext, Context};
use serenity::http::Http;
use serenity::prelude::Mentionable;
use shared::localization::{get_language_identifier, Loader, USABLE_LOCALES};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

#[slash_command(
	name = "join", desc = "Join the voice channel.",
	command_type = SubCommand(parent = "music"),
	contexts = [Guild],
	install_contexts = [Guild],
)]
async fn join_command(self_: JoinCommand) -> Result<EmbedsContents<'_>> {
	self_.defer().await?;
	let ctx = self_.get_ctx().clone();
	let bot_data = ctx.data::<BotData>().clone();
	let command_interaction = self_.get_command_interaction().clone();

	let (_, embed_content) = join(ctx, bot_data, command_interaction).await?;
	let embed = embed_content.clone();

	Ok(embed)
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
pub async fn join<'a>(
	ctx: Context, bot_data: Arc<BotData>, command_interaction: CommandInteraction,
) -> Result<(bool, EmbedsContents<'a>)> {
	// Retrieve the guild ID from the command interaction
	let guild_id_str = match command_interaction.guild_id {
		Some(id) => id.to_string(),
		None => String::from("0"),
	};
	let db_connection = bot_data.db_connection.clone();

	// Load the localized strings
	let lang_id = get_language_identifier(guild_id_str, db_connection).await;

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
				let embed_content =
					EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "music_join-title"))
						.description(USABLE_LOCALES.lookup(&lang_id, "music_join-error_no_voice"));
				let embed_contents =
					EmbedsContents::new(CommandType::Followup, vec![embed_content]);
				return Ok((false, embed_contents));
			},
		}
	};

	// Create the embed content outside the non-Send guild reference scope
	let mut embed_content = EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "music_join-title"));

	if lava_client
		.get_player_context(lavalink_rs::model::GuildId::from(guild_id.get()))
		.is_none()
	{
		let handler = manager.join_gateway(guild_id, connect_to.clone()).await;

		let (result, return_data) = match handler {
			Ok((connection_info, _)) => {
				lava_client
					.create_player_context_with_data::<(ChannelId, Arc<Http>)>(
						lavalink_rs::model::GuildId::from(guild_id.get()),
						lavalink_rs::model::player::ConnectionInfo {
							endpoint: connection_info.endpoint,
							token: connection_info.token,
							session_id: connection_info.session_id,
							channel_id: Some(ChannelId::from(connect_to)),
						},
						Arc::new((ChannelId(channel_id.get()), ctx.http.clone())),
					)
					.await?;

				let mut args: HashMap<Cow<'static, str>, FluentValue> = HashMap::new();
				args.insert(
					Cow::Borrowed("var0"),
					FluentValue::from(connect_to.mention().to_string()),
				);

				embed_content = embed_content.description(USABLE_LOCALES.lookup_with_args(
					&lang_id,
					"music_join-success",
					&args,
				));
				let embed_contents =
					EmbedsContents::new(CommandType::Followup, vec![embed_content]);

				(true, embed_contents)
			},
			Err(why) => {
				let mut args: HashMap<Cow<'static, str>, FluentValue> = HashMap::new();
				args.insert(Cow::Borrowed("var0"), FluentValue::from(why.to_string()));

				embed_content = embed_content.description(USABLE_LOCALES.lookup_with_args(
					&lang_id,
					"music_join-error_joining",
					&args,
				));
				let embed_contents =
					EmbedsContents::new(CommandType::Followup, vec![embed_content]);

				(false, embed_contents)
			},
		};
		return Ok((result, return_data));
	};
	let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

	Ok((false, embed_contents))
}

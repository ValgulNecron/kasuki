//! Module implementing the `InfoCommand` structure and its functionality.
use crate::command::command::Command;
use crate::command::embed_content::{
	ButtonV1, CommandType, ComponentVersion, ComponentVersion1, CreateFooter, EmbedContent,
	EmbedsContents,
};
use crate::constant::{APP_VERSION, LIBRARY};
use crate::database::prelude::UserColor;
use crate::event_handler::BotData;
use crate::get_url;
use crate::structure::message::bot::info::load_localization_info;
use anyhow::{Result, anyhow};
use sea_orm::EntityTrait;
use serenity::all::{
	ButtonStyle, CommandInteraction, Context as SerenityContext, CreateActionRow, CreateButton,
};
use std::borrow::Cow;

/// The `InfoCommand` struct is used to encapsulate information needed to execute a command
/// within a Discord bot using the `serenity` library. This struct provides the necessary
/// context and interaction details for handling a specific command event.
///
/// Fields:
/// - `ctx: SerenityContext`
///     - Represents the context of the bot's runtime at the time of the interaction.
///       It contains information such as the shard information, cache, HTTP client, and more.
///       This is used to interact with and query the Discord API, send messages, and perform
///       other operations within the bot's lifecycle.
///
/// - `command_interaction: CommandInteraction`
///     - Represents the interaction details when a user invokes a command on the bot.
///       This contains the source of the command interaction, user information, command data,
///       and the ability to respond to the interaction.
///
pub struct InfoCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for InfoCommand {
	/// Retrieves a reference to the `SerenityContext` associated with the current instance.
	///
	/// # Returns
	/// A reference to the `SerenityContext` stored within the current instance.
	///
	/// # Examples
	/// ```
	/// let context = instance.get_ctx();
	/// // Use `context` for operations involving the Serenity framework.
	/// ```
	///
	/// This method is useful for accessing the `ctx` field when performing operations
	/// that require a reference to the bot's context in the Serenity framework.
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` associated with the current instance.
	///
	/// # Returns
	/// A shared reference to the `CommandInteraction` field.
	///
	/// # Example
	/// ```rust
	/// let command_interaction = instance.get_command_interaction();
	/// // Perform operations with `command_interaction`
	/// ```
	///
	/// # Note
	/// This method does not take ownership of the `CommandInteraction` object,
	/// instead it provides a reference, allowing read-only access.
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Retrieves a vector of `EmbedContent` containing detailed information about the bot.
	///
	/// This asynchronous function prepares a rich embed structure with metadata about the bot,
	/// such as its name, ID, creation date, shard and server details, user count, library details,
	/// and various links for external resources like GitHub, official website, and Discord.
	///
	/// #### Additional Actions:
	/// - Buttons for actions such as visiting the GitHub repository, joining the official Discord,
	///   or adding the bot to a server are included in the embed's components.
	///
	/// # Returns
	/// - A `Result` with a `Vec<EmbedContent<'_, '_>>` on success.
	/// - On failure, it returns any encountered error encapsulated in the `Result`.
	///
	/// # Metadata Included in Embed:
	/// - **Bot Name**: The name of the bot.
	/// - **Bot ID**: The unique identifier of the bot.
	/// - **Version**: The current application version.
	/// - **Shard Count**: The total number of shards.
	/// - **Shard**: The current shard's ID.
	/// - **User Count**: The total number of users the bot interacts with.
	/// - **Server Count**: The number of servers (guilds) the bot is in.
	/// - **Creation Date**: The timestamp of the bot's creation.
	/// - **Library**: The library used by the bot.
	/// - **App Installation Count**: The estimated number of bot installations.
	///
	/// # Asynchronous Dependencies:
	/// - Requires fetching data from Discord's HTTP API to retrieve bot details.
	/// - Fetches localization information from the database based on the guild ID.
	/// - Retrieves application information via `get_current_application_info`.
	/// - Connects to an `sea_orm` database to fetch additional data like user counts.
	///
	/// # Example Usage:
	/// ```rust
	/// let embed_contents = self.get_contents().await?;
	/// for embed in embed_contents {
	///     println!("{:?}", embed);
	/// }
	/// ```
	///
	/// # Errors:
	/// - The function returns an error if any of the following occur:
	///   - Failure to fetch localization information (`load_localization_info`).
	///   - Issues connecting to or querying the database.
	///   - Failure in interacting with Discord's API or retrieving bot details.
	///   - Missing bot icon.
	///
	/// # Notes:
	/// - The bot's avatar is retrieved in either a `.gif` or `.webp` format based on whether it is animated.
	/// - Buttons for adding the bot or beta bot include pre-configured authorization URLs.
	///
	/// # Dependencies:
	/// - `sea_orm`: ORM for database connection and queries.
	/// - `discord_http`: API interaction for bot and guild information.
	/// - Localization data loaded per guild via `load_localization_info`.
	///
	/// # Components:
	/// - **GitHub Repository Link**: A button directing to the bot's code repository.
	/// - **Official Website Link**: A button linking to the bot's official webpage.
	/// - **Official Discord Server**: A button linking to join the community server.
	/// - **Invitation Links**: Buttons for adding stable and beta versions of the bot.
	async fn get_contents(&self) -> Result<EmbedsContents> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();
		let config = bot_data.config.clone();

		// Retrieve the guild ID from the command interaction
		let guild_id = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};

		// Load the localized information strings
		let info_localised = load_localization_info(guild_id, config.db.clone()).await?;

		// Retrieve various details about the bot and the server
		let shard_count = ctx.cache.shard_count();

		let shard = ctx.shard_id.to_string();

		let connection = sea_orm::Database::connect(get_url(config.db.clone())).await?;

		let user_count = UserColor::find().all(&connection).await?.len();

		let bot = ctx.http.get_current_application_info().await?;

		let bot_name = bot.name.to_string();

		let bot_id = bot.id.to_string();

		let creation_date = format!("<t:{}:F>", bot.id.created_at().unix_timestamp());

		let server_count = ctx.cache.guild_count();

		let app_guild_count = bot.approximate_guild_count.unwrap_or_default() as usize;

		let guild_count = if server_count > app_guild_count {
			server_count
		} else {
			app_guild_count
		};

		let app_installation_count =
			bot.approximate_user_install_count.unwrap_or_default() as usize;

		// Retrieve the bot's avatar
		let bot_icon = bot.icon.ok_or(anyhow!("No bot icon"))?;

		let avatar = if bot_icon.is_animated() {
			format!(
				"https://cdn.discordapp.com/icons/{}/{}.gif?size=1024",
				bot_id, bot_icon
			)
		} else {
			format!(
				"https://cdn.discordapp.com/icons/{}/{}.webp?size=1024",
				bot_id, bot_icon
			)
		};

		let lib = LIBRARY.to_string();

		let mut buttons = vec![];

		// Add buttons for various actions

		buttons.push(
			ButtonV1::new(info_localised.button_see_on_github)
				.url("https://github.com/ValgulNecron/kasuki".to_string())
				.style(ButtonStyle::Primary),
		);

		buttons.push(
			ButtonV1::new(info_localised.button_official_website)
				.url("https://kasuki.moe/".to_string())
				.style(ButtonStyle::Primary),
		);

		buttons.push(
			ButtonV1::new(info_localised.button_official_discord)
				.url("https://discord.gg/JwdYfnXaeK".to_string())
				.style(ButtonStyle::Primary),
		);

		buttons.push(
			ButtonV1::new(info_localised.button_add_the_bot)
				.url("https://discord.com/api/oauth2/authorize?client_id=923286536445894697&permissions=395677134144&scope=bot".to_string())
				.style(ButtonStyle::Success),
        );

		buttons.push(
			ButtonV1::new(info_localised.button_add_the_beta_bot)
				.url("https://discord.com/api/oauth2/authorize?client_id=1122304053620260924&permissions=395677134144&scope=bot".to_string())
				.style(ButtonStyle::Secondary),
        );

		let embed_content = EmbedContent::new(info_localised.title)
			.description(info_localised.desc)
			.thumbnail(avatar)
			.fields(vec![
				(info_localised.bot_name, bot_name, true),
				(info_localised.bot_id, bot_id, true),
				(info_localised.version, String::from(APP_VERSION), true),
				(info_localised.shard_count, shard_count.to_string(), true),
				(info_localised.shard, shard, true),
				(info_localised.user_count, user_count.to_string(), true),
				(info_localised.server_count, guild_count.to_string(), true),
				(info_localised.creation_date, creation_date, true),
				(info_localised.library, lib, true),
				(
					info_localised.app_installation_count,
					app_installation_count.to_string(),
					true,
				),
			])
			.footer(CreateFooter::new(info_localised.footer));

		let embed_contents = EmbedsContents::new(CommandType::First, vec![embed_content])
			.action_row(ComponentVersion::V1(ComponentVersion1::Buttons(buttons)));

		Ok(embed_contents)
	}
}

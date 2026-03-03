//! The `SteamGameInfoCommand` struct represents a Discord Bot command to fetch and display
//! information about a game from Steam. This command generates a detailed embed message
//! containing details such as price, platforms, developers, publishers, and more.
//!
//! # Fields
//!
//! * `ctx`: The [`SerenityContext`](serenity::all::Context) holding the shared mutable and immutable state of the bot.
//! * `command_interaction`: The interaction object containing event data about a user command.
//!
//! # Traits Implemented
//!
//! ## `Command`
//!
//! - `get_ctx(&self) -> &SerenityContext`: Returns the Discord bot context (`SerenityContext`).
//! - `get_command_interaction(&self) -> &CommandInteraction`: Returns the user interaction object (`CommandInteraction`).
//! - `get_contents(&self) -> Result<Vec<EmbedContent<'_, '_>>>`: Generates the content for the embed
//!   displaying detailed game information after fetching it from the Steam API.
//!
//! # Example
//!
//! ```
//! impl Command for SteamGameInfoCommand {
//!     async fn get_contents(&self) -> Result<Vec<EmbedContent<'_, '_>>> {
//!         // Retrieves and processes game information from Steam.
//!         // Generates and returns a detailed embed with this data.
//!     }
//! }
//! ```
//!
//! ## Supported Game Information Fields
//!
//! 1. **Price**: Displays the game price. Indicates if the game is free or provides formatted price and discounts.
//! 2. **Platforms**: Shows platform support (Windows, macOS, Linux).
//! 3. **Website**: Displays the official website of the game (if available).
//! 4. **Age Requirement**: Indicates the minimum required age.
//! 5. **Release Date**: Specifies whether the game is "Coming Soon" or its release date.
//! 6. **Developers and Publishers**: Lists developers and publishers of the game.
//! 7. **App Type and Categories**: Specifies the type of the app (e.g., game, software) and categorized features.
//! 8. **Languages**: Lists supported languages.
//!
//! ## Example Embed Output
//!
//! ```text
//! Title: Game Name
//! Description: A short description about the game.
//! Fields:
//! - Price: Free/$15.99 (-40% off)
//! - Release Date: November 20, 2023
//! - Developers: Developer Name
//! - Publishers: Publisher Name
//! - Platforms:
//!     Windows: True
//!     Mac: False
//!     Linux: True
//! - Languages: English, Spanish, French
//! ```
//!
//! ## Error Handling
//!
//! If the game name or ID is invalid, the command will return a meaningful error using `anyhow::Error`.
//!
//! ## Dependencies
//!
//! This command utilizes the following helper functions, modules, and traits:
//! - `convert_flavored_markdown::convert_steam_to_discord_flavored_markdown`: Converts Steam-formatted content to Discord-specific markdown.
//! - `get_option_map_string_subcommand`: Parses user-provided input (game name or ID) from the command interaction.
//! - `steam_game_info::load_localization_steam_game_info`: Loads localized strings for embed field names.
//! - `SteamGameWrapper`: Abstraction to handle Steam API game data fetching based on ID or name.
//!
//! # Related Functions
//!
//! ## `get_steam_game`
//!
//! This helper function is used to retrieve Steam game data and construct a `SteamGameWrapper`.
//!
//! ### Parameters
//!
//! - `apps`: A synchronized map (`Arc<RwLock<HashMap>>`) containing application data for enhanced search functionality.
//! - `command_interaction`: The interaction object containing user command input.
//! - `config`: Reference to bot configuration containing database and other settings.
//!
//! ### Returns
//!
//! Returns a `SteamGameWrapper` containing the Steam game data fetched by ID or search query.
//!
//! ### Example Usage
//!
//! ```
//! async fn get_contents(&self) -> Result<Vec<EmbedContent<'_, '_>>> {
//!     let data = get_steam_game(bot_data.apps.clone(), interaction.clone(), bot_data.config.clone()).await?;
//!     // Process and use the retrieved SteamGameWrapper.
//! }
//! ```
use crate::command::command::CommandRun;
use crate::command::embed_content::{EmbedContent, EmbedsContents};
use crate::event_handler::BotData;
use crate::helper::convert_flavored_markdown::convert_steam_to_discord_flavored_markdown;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::structure::run::game::steam_game::{Platforms, SteamGameWrapper};
use anyhow::{anyhow, Result};
use kasuki_macros::slash_command;
use sea_orm::DatabaseConnection;
use serenity::all::{CommandInteraction, Context as SerenityContext, GuildId};
use shared::localization::{get_language_identifier, Loader, USABLE_LOCALES};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[slash_command(
	name = "game", desc = "Get info of a steam game.",
	command_type = SubCommand(parent = "steam"),
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
	args = [(name = "game_name", desc = "Name of the steam game you want info of.", arg_type = String, required = true, autocomplete = true)],
)]
async fn steam_game_info_command(self_: SteamGameInfoCommand) -> Result<EmbedsContents<'_>> {
	let ctx = self_.get_ctx();
	let bot_data = ctx.data::<BotData>().clone();
	let db_connection = bot_data.db_connection.clone();
	let data = get_steam_game(
		bot_data.apps.clone(),
		self_.command_interaction.clone(),
		db_connection,
	)
	.await?;
	let command_interaction = self_.get_command_interaction();

	let guild_id = command_interaction
		.guild_id
		.unwrap_or(GuildId::from(0))
		.to_string();
	let db_connection = bot_data.db_connection.clone();

	let lang_id = get_language_identifier(guild_id.clone(), db_connection).await;

	let game = data.data;

	let mut fields = Vec::new();

	// Determine the price field based on whether the game is free or not
	let field1 = if game.is_free.unwrap() {
		(
			USABLE_LOCALES.lookup(&lang_id, "game_steam_game_info-field1"),
			USABLE_LOCALES.lookup(&lang_id, "game_steam_game_info-free"),
			true,
		)
	} else {
		match game.price_overview {
			Some(price) => {
				let price = format!(
					"{} {}",
					price.final_formatted.unwrap_or_default(),
					price.discount_percent.unwrap_or_default()
				);

				(
					USABLE_LOCALES.lookup(&lang_id, "game_steam_game_info-field1"),
					convert_steam_to_discord_flavored_markdown(price),
					true,
				)
			},
			None => (
				USABLE_LOCALES.lookup(&lang_id, "game_steam_game_info-field1"),
				USABLE_LOCALES.lookup(&lang_id, "game_steam_game_info-tba"),
				true,
			),
		}
	};

	fields.push(field1);

	let platforms = match game.platforms {
		Some(platforms) => platforms,
		_ => Platforms {
			windows: None,
			mac: None,
			linux: None,
		},
	};

	if let Some(website) = game.website {
		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "game_steam_game_info-website"),
			convert_steam_to_discord_flavored_markdown(website),
			true,
		));
	}

	if let Some(required_age) = game.required_age {
		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "game_steam_game_info-required_age"),
			required_age.to_string(),
			true,
		));
	}

	// Determine the release date field based on whether the game is coming soon or not
	let field2 = if game.release_date.clone().unwrap().coming_soon {
		match game.release_date.unwrap().date {
			Some(date) => (
				USABLE_LOCALES.lookup(&lang_id, "game_steam_game_info-field2"),
				convert_steam_to_discord_flavored_markdown(date),
				true,
			),
			None => (
				USABLE_LOCALES.lookup(&lang_id, "game_steam_game_info-field2"),
				USABLE_LOCALES.lookup(&lang_id, "game_steam_game_info-coming_soon"),
				true,
			),
		}
	} else {
		(
			USABLE_LOCALES.lookup(&lang_id, "game_steam_game_info-field2"),
			convert_steam_to_discord_flavored_markdown(game.release_date.unwrap().date.unwrap()),
			true,
		)
	};

	fields.push(field2);

	// Add the developers field if it exists
	if let Some(dev) = game.developers {
		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "game_steam_game_info-field3"),
			convert_steam_to_discord_flavored_markdown(dev.join(", ")),
			true,
		))
	}

	// Add the publishers field if it exists
	if let Some(publishers) = game.publishers {
		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "game_steam_game_info-field4"),
			convert_steam_to_discord_flavored_markdown(publishers.join(", ")),
			true,
		))
	}

	// Add the app type field if it exists
	if let Some(app_type) = game.app_type {
		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "game_steam_game_info-field5"),
			convert_steam_to_discord_flavored_markdown(app_type),
			true,
		))
	}

	// Add the supported languages field if it exists
	if let Some(game_lang) = game.supported_languages {
		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "game_steam_game_info-field6"),
			convert_steam_to_discord_flavored_markdown(game_lang),
			true,
		))
	}

	let win = platforms.windows.unwrap_or(false);

	let mac = platforms.mac.unwrap_or(false);

	let linux = platforms.linux.unwrap_or(false);

	fields.push((
		USABLE_LOCALES.lookup(&lang_id, "game_steam_game_info-win"),
		win.to_string(),
		true,
	));

	fields.push((
		USABLE_LOCALES.lookup(&lang_id, "game_steam_game_info-mac"),
		mac.to_string(),
		true,
	));

	fields.push((
		USABLE_LOCALES.lookup(&lang_id, "game_steam_game_info-linux"),
		linux.to_string(),
		true,
	));

	// Add the categories field if it exists
	if let Some(categories) = game.categories {
		let descriptions: Vec<String> = categories
			.into_iter()
			.filter_map(|category| category.description)
			.collect();

		let joined_descriptions =
			convert_steam_to_discord_flavored_markdown(descriptions.join(", "));

		fields.push((
			USABLE_LOCALES.lookup(&lang_id, "game_steam_game_info-field7"),
			joined_descriptions,
			false,
		))
	}

	let embed_content = EmbedContent::new(game.name.unwrap())
		.description(convert_steam_to_discord_flavored_markdown(
			game.short_description.unwrap_or_default(),
		))
		.fields(fields)
		.url(format!(
			"https://store.steampowered.com/app/{}",
			game.steam_appid.unwrap()
		))
		.images_url(game.header_image.unwrap());

	let embed_contents = EmbedsContents::new(vec![embed_content]);

	Ok(embed_contents)
}

/// Fetches a Steam game based on user input from a command interaction.
///
/// # Arguments
///
/// * `apps` - An `Arc<RwLock<HashMap<String, u128>>>` containing cached app data. This is used for
///   searching Steam games by name.
/// * `command_interaction` - A `CommandInteraction` containing the details of the user's command. It
///   includes necessary information such as options and the associated guild.
/// * `config` - An `Arc<Config>` that provides access to configuration and database connection details.
///
/// # Returns
///
/// This function returns a `Result` containing the `SteamGameWrapper` upon success, or an error if
/// the operation fails.
///
/// # Behavior
///
/// The function first retrieves the `guild_id` from the `command_interaction` or defaults it to `0`
/// if unavailable. It then extracts the user-provided `game_name` option from the command's parameters.
///
/// If the `game_name` can be parsed as a numeric ID (`i128`), the game is fetched by its Steam App ID
/// using `SteamGameWrapper::new_steam_game_by_id`. Otherwise, the game is searched by name using
/// `SteamGameWrapper::new_steam_game_by_search`.
///
/// The resulting `SteamGameWrapper` is then returned.
///
/// # Errors
///
/// This function will return an error if:
/// - The `game_name` option is missing from the command interaction.
/// - Any error occurs while fetching the game data (e.g., network or database issues).
///
/// # Example
///
/// ```ignore
/// let command_interaction = ...; // Received from a user interaction
/// let steam_app_cache = Arc::new(RwLock::new(HashMap::new()));
/// let config = Arc::new(config_instance);
///
/// match get_steam_game(steam_app_cache, command_interaction, config).await {
///     Ok(steam_game) => println!("Game found: {:?}", steam_game),
///     Err(e) => eprintln!("Failed to fetch Steam game: {:?}", e),
/// }
/// ```
async fn get_steam_game(
	apps: Arc<RwLock<HashMap<String, u128>>>, command_interaction: CommandInteraction,
	db_connection: Arc<DatabaseConnection>,
) -> Result<SteamGameWrapper> {
	let guild_id = command_interaction
		.guild_id
		.unwrap_or(GuildId::from(0))
		.to_string();

	let map = get_option_map_string_subcommand(&command_interaction);

	let value = map
		.get(&String::from("game_name"))
		.ok_or(anyhow!("No option for game_name"))?;

	let data: SteamGameWrapper = if value.parse::<i128>().is_ok() {
		SteamGameWrapper::new_steam_game_by_id(value.parse().unwrap(), guild_id, db_connection)
			.await?
	} else {
		SteamGameWrapper::new_steam_game_by_search(value, guild_id, apps, db_connection).await?
	};

	Ok(data)
}

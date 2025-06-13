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
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::Arc;

use crate::command::command::Command;
use crate::command::embed_content::{CommandType, EmbedContent, EmbedsContents};
use crate::config::Config;
use crate::event_handler::BotData;
use crate::helper::convert_flavored_markdown::convert_steam_to_discord_flavored_markdown;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::structure::message::game::steam_game_info::load_localization_steam_game_info;
use crate::structure::run::game::steam_game::{Platforms, SteamGameWrapper};
use serenity::all::{CommandInteraction, Context as SerenityContext, GuildId};
use tokio::sync::RwLock;

/// A struct representing a command to fetch Steam game information.
///
/// This struct encapsulates the necessary context and interaction
/// metadata required to handle a command that retrieves information
/// about a Steam game.
///
/// # Fields
///
/// * `ctx` - The Discord bot's context, represented by `SerenityContext`. This
///   provides access to information and resources for interacting with the
///   bot's environment and its events.
/// * `command_interaction` - The interaction metadata for the specific command
///   invocation, represented by `CommandInteraction`. This contains details about
///   the user's input and can be used to respond back to the user.
///
/// # Example
/// ```ignore
/// use your_crate::SteamGameInfoCommand;
///
/// let steam_game_info_command = SteamGameInfoCommand {
///     ctx: serenity_context_variable,
///     command_interaction: command_interaction_variable,
/// };
///
/// // Use `steam_game_info_command` to handle the Steam game information request.
/// ```
pub struct SteamGameInfoCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for SteamGameInfoCommand {
	/// Retrieves a reference to the `SerenityContext` associated with the current instance.
	///
	/// # Returns
	/// * `&SerenityContext` - A reference to the `SerenityContext` stored within the struct.
	///
	/// # Example
	/// ```
	/// let context = instance.get_ctx();
	/// // Use the retrieved SerenityContext reference for further operations
	/// ```
	///
	/// This method provides read-only access to the stored `SerenityContext`.
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` associated with the current instance.
	///
	/// # Returns
	/// A reference to the `CommandInteraction` object, which provides details about the command interaction.
	///
	/// # Example
	/// ```
	/// let command_interaction = instance.get_command_interaction();
	/// ```
	///
	/// This method can be used to access the command interaction for further processing or inspection.
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Asynchronously retrieves the content of a Steam game and formats it into Discord-styled embed content.
	///
	/// # Returns
	///
	/// Returns a `Result` containing a `Vec` of `EmbedContent` objects which represent the structured embed data for Discord.
	/// On an error, it returns the associated error wrapped in the `Err`.
	///
	/// # Details
	///
	/// This function interacts with the Steam API to retrieve game information and transforms it into an embed format designed
	/// for Discord interactions. It includes the following fields:
	/// - Price: Shows the price of the game or indicates if the game is free or To Be Announced.
	/// - Platforms: Indicates availability of the game for Windows, macOS, and Linux.
	/// - Website: Adds the official game website, if available.
	/// - Age Restriction: Displays an age restriction, if specified.
	/// - Release Date: Displays the release date or notes if the game is "Coming Soon."
	/// - Developers and Publishers: Lists the developers and publishers of the game.
	/// - Application Type: Shows the type of the Steam application (e.g., game, software).
	/// - Supported Languages: Lists the languages the game supports.
	/// - Categories: Displays the categories or metadata tags associated with the game.
	///
	/// Additionally, fields like the game name, description, image, and Steam Store link are also included in the embed.
	///
	/// # Steps Involved
	///
	/// 1. Retrieve context and relevant dependencies including Bot data, command interaction, and configuration.
	/// 2. Retrieve Steam game data using `get_steam_game`.
	/// 3. Load localised strings based on the guild ID for embedding content in the user's locale.
	/// 4. Construct fields for the embed:
	///     - Conditionally formats fields such as price, platforms, and release dates based on game attributes (e.g., `free`, `coming soon`).
	///     - Handles objects like developers, publishers, and categories, ensuring they format as Markdown where appropriate.
	///     - Converts strings to Discord-compatible Markdown using `convert_steam_to_discord_flavored_markdown`.
	/// 5. Constructs an `EmbedContent` object with:
	///     - A descriptive title.
	///     - A cleaned and Markdown-compatible short description.
	///     - Dynamically built fields with game metadata.
	///     - Steam game URL and image link.
	/// 6. Returns the compiled embed as a `Vec<EmbedContent>`.
	///
	/// # Errors
	///
	/// This function will return an `Err` in the following scenarios:
	/// - Failure to access game data from the Steam API.
	/// - Localization loading errors.
	/// - Required fields in the `game` object being `None` when expected (e.g., missing name, Steam App ID).
	///
	/// # Example Usage
	///
	/// ```rust
	/// let steam_embed_contents = some_instance.get_contents().await?;
	/// // Use `steam_embed_contents` for Discord messages or further processing.
	/// ```
	///
	/// # Dependencies
	///
	/// - Requires `get_steam_game` for retrieving game information from Steam.
	/// - Uses `load_localization_steam_game_info` to load embed localization for the guild-specific language.
	/// - Utilizes helper functions such as `convert_steam_to_discord_flavored_markdown` to format raw strings.
	///
	/// # Notes
	///
	/// - Ensure the `BotData` context is configured correctly with necessary Steam API credentials and application data.
	/// - This function assumes certain responses from the Steam API match the expected schema; modifications to the schema may require code updates.
	async fn get_contents<'a>(&'a self) -> anyhow::Result<EmbedsContents<'a>> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let data = get_steam_game(
			bot_data.apps.clone(),
			self.command_interaction.clone(),
			bot_data.config.clone(),
		)
		.await?;
		let command_interaction = self.get_command_interaction();
		let config = bot_data.config.clone();

		let guild_id = command_interaction
			.guild_id
			.unwrap_or(GuildId::from(0))
			.to_string();

		let steam_game_info_localised =
			load_localization_steam_game_info(guild_id.clone(), config.db.clone()).await?;

		let game = data.data;

		let mut fields = Vec::new();

		// Determine the price field based on whether the game is free or not
		let field1 = if game.is_free.unwrap() {
			(
				steam_game_info_localised.field1,
				steam_game_info_localised.free,
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
						steam_game_info_localised.field1,
						convert_steam_to_discord_flavored_markdown(price),
						true,
					)
				},
				None => (
					steam_game_info_localised.field1,
					steam_game_info_localised.tba,
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
				steam_game_info_localised.website,
				convert_steam_to_discord_flavored_markdown(website),
				true,
			));
		}

		if let Some(required_age) = game.required_age {
			fields.push((
				steam_game_info_localised.required_age,
				required_age.to_string(),
				true,
			));
		}

		// Determine the release date field based on whether the game is coming soon or not
		let field2 = if game.release_date.clone().unwrap().coming_soon {
			match game.release_date.unwrap().date {
				Some(date) => (
					steam_game_info_localised.field2,
					convert_steam_to_discord_flavored_markdown(date),
					true,
				),
				None => (
					steam_game_info_localised.field2,
					steam_game_info_localised.coming_soon,
					true,
				),
			}
		} else {
			(
				steam_game_info_localised.field2,
				convert_steam_to_discord_flavored_markdown(
					game.release_date.unwrap().date.unwrap(),
				),
				true,
			)
		};

		fields.push(field2);

		// Add the developers field if it exists
		if let Some(dev) = game.developers {
			fields.push((
				steam_game_info_localised.field3,
				convert_steam_to_discord_flavored_markdown(dev.join(", ")),
				true,
			))
		}

		// Add the publishers field if it exists
		if let Some(publishers) = game.publishers {
			fields.push((
				steam_game_info_localised.field4,
				convert_steam_to_discord_flavored_markdown(publishers.join(", ")),
				true,
			))
		}

		// Add the app type field if it exists
		if let Some(app_type) = game.app_type {
			fields.push((
				steam_game_info_localised.field5,
				convert_steam_to_discord_flavored_markdown(app_type),
				true,
			))
		}

		// Add the supported languages field if it exists
		if let Some(game_lang) = game.supported_languages {
			fields.push((
				steam_game_info_localised.field6,
				convert_steam_to_discord_flavored_markdown(game_lang),
				true,
			))
		}

		let win = platforms.windows.unwrap_or(false);

		let mac = platforms.mac.unwrap_or(false);

		let linux = platforms.linux.unwrap_or(false);

		fields.push((steam_game_info_localised.win, win.to_string(), true));

		fields.push((steam_game_info_localised.mac, mac.to_string(), true));

		fields.push((steam_game_info_localised.linux, linux.to_string(), true));

		// Add the categories field if it exists
		if let Some(categories) = game.categories {
			let descriptions: Vec<String> = categories
				.into_iter()
				.filter_map(|category| category.description)
				.collect();

			let joined_descriptions =
				convert_steam_to_discord_flavored_markdown(descriptions.join(", "));

			fields.push((steam_game_info_localised.field7, joined_descriptions, false))
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

		let embed_contents = EmbedsContents::new(CommandType::Followup, vec![embed_content]);

		Ok(embed_contents)
	}
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
	config: Arc<Config>,
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
		SteamGameWrapper::new_steam_game_by_id(value.parse().unwrap(), guild_id, config.db.clone())
			.await?
	} else {
		SteamGameWrapper::new_steam_game_by_search(value, guild_id, apps, config.db.clone()).await?
	};

	Ok(data)
}

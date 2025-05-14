//! This module defines functionality for the `AddActivityCommand` which handles adding
//! activities related to an anime to a database with support for localization and discord webhook notifications.
//!
//! The key components include:
//! - The `AddActivityCommand` struct implementing the `Command` trait to encapsulate the command's behavior.
//! - Several helper functions such as `resize_image`, `calculate_crop_params`, and `check_if_activity_exist`
//!   to assist with image management, database queries, and other operations.
//! - Integration with SeaORM for database interactions such as inserting or checking activity data.
//! - Calling Anilist GraphQL API to retrieve anime media information.
//!
//! Dependencies:
//! - Various crates such as `anyhow`, `reqwest`, `sea_orm`, `image`, `base64`, `chrono`, `serenity`, and more.
//!
//!
//! # Structures
//!
//! ## `AddActivityCommand`
//!
//! Represents a command to add anime-related activity to the database.
//! Contains the following fields:
//! - `ctx`: The Discord bot context for accessing shared data and resources.
//! - `command_interaction`: The interaction information triggered by the user.
//!
//! Implements the `Command` trait's required functions:
//! - `get_ctx`: Provides access to the current bot context.
//! - `get_command_interaction`: Provides the command interaction context.
//! - `get_contents`: The main functionality to process the command.
//!
//! # Helpers
//!
//! ## `resize_image`
//!
//! Asynchronously resizes an image to a 128x128 square while preserving its aspect ratio. When resizing,
//! it determines the appropriate crop region based on the dimensions of the original image.
//!
//! Parameters:
//! - `image_bytes`: Byte representation of the source image.
//!
//! Returns:
//! - A `Result` containing a cursor pointing to the resized image in JPEG format.
//!
//! ## `calculate_crop_params`
//!
//! Calculates the cropping parameters (starting x, starting y, and size of the square region) for resizing an image.
//!
//! Parameters:
//! - `width`: The width of the original image.
//! - `height`: The height of the original image.
//!
//! Returns:
//! - A tuple with cropping parameters `(crop_x, crop_y, square_size)`.
//!
//! ## `check_if_activity_exist`
//!
//! Asynchronously checks if an anime activity already exists in a server's database.
//!
//! Parameters:
//! - `anime_id`: The ID of the anime to check existence for.
//! - `server_id`: The server's unique identifier.
//! - `config`: Database configuration details.
//!
//! Returns:
//! - `true` if an anime activity exists, `false` otherwise.
//!
//! # Usage Workflow
//!
//! 1. User triggers the `AddActivityCommand` with necessary anime information.
//! 2. The command handler retrieves user input, such as the anime name and delay settings.
//! 3. Queries the Anilist API for anime details and checks if the activity already exists in the database.
//! 4. If the activity does not exist:
//!    - Performs necessary image processing (resize and encoding).
//!    - Creates and sends a Discord webhook with activity information.
//!    - Stores the activity in the database.
//! 5. Sends feedback to the user either confirming the successful addition or reporting failure.
use anyhow::{Result, anyhow};
use std::io::{Cursor, Read};
use std::sync::Arc;

use crate::command::command::{Command, EmbedType};
use crate::command::command::{CommandRun, EmbedContent};
use crate::config::DbConfig;
use crate::database::activity_data;
use crate::database::activity_data::Column;
use crate::database::prelude::ActivityData;
use crate::event_handler::BotData;
use crate::get_url;
use crate::helper::get_option::subcommand_group::get_option_map_string_subcommand_group;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::helper::trimer::trim_webhook;
use crate::structure::message::admin::anilist::add_activity::load_localization_add_activity;
use crate::structure::run::anilist::minimal_anime::{
	Media, MediaTitle, MinimalAnimeId, MinimalAnimeIdVariables, MinimalAnimeSearch,
	MinimalAnimeSearchVariables,
};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use base64::read::DecoderReader;
use bytes::Bytes;
use chrono::Utc;
use cynic::{GraphQlResponse, QueryBuilder};
use image::imageops::FilterType;
use image::{GenericImageView, ImageFormat, guess_format};
use moka::future::Cache;
use reqwest::get;
use sea_orm::ActiveValue::Set;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use serde_json::json;
use serenity::all::{
	ChannelId, CommandInteraction, Context as SerenityContext, CreateAttachment, EditWebhook,
	GenericChannelId,
};
use tokio::sync::RwLock;
use tracing::trace;

/// A struct representing the `AddActivityCommand`, which encapsulates the context and interaction
/// details required for handling the "Add Activity" command in a Discord bot.
///
/// # Fields
///
/// * `ctx` - The `SerenityContext`, providing access to the state and resources of the running bot.
/// * `command_interaction` - The `CommandInteraction` containing the interaction information
///   (e.g., user input and metadata) for the "Add Activity" command.
///
/// # Usage
///
/// This struct is typically used to store the necessary data for executing an "Add Activity"
/// command initiated by a user in a Discord server. It can later be processed to perform the
/// required operation.
///
/// # Example
///
/// ```rust
/// let add_activity = AddActivityCommand {
///     ctx: serenity_context,
///     command_interaction: interaction,
/// };
/// ```
pub struct AddActivityCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for AddActivityCommand {
	/// Retrieves a reference to the `SerenityContext` associated with the current instance.
	///
	/// # Returns
	///
	/// A reference to the `SerenityContext` (`&SerenityContext`) which provides access to the
	/// Discord API and various utilities for interacting with Discord via the serenity framework.
	///
	/// # Example
	///
	/// ```rust
	/// let context = instance.get_ctx();
	/// // Use the `context` to interact with Discord or retrieve additional data
	/// ```
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` instance associated with the current object.
	///
	/// # Returns
	/// A reference to the `CommandInteraction` instance (`&CommandInteraction`) stored within the object.
	///
	/// # Example
	/// ```rust
	/// let interaction = my_object.get_command_interaction();
	/// // Use the interaction object as needed
	/// ```
	///
	/// # Notes
	/// - This method does not take any parameters.
	/// - The returned reference borrows the internal `CommandInteraction` instance and follows Rust's borrowing rules.
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Asynchronously processes the functionality for retrieving and managing content related to anime activities.
	///
	/// # Returns
	/// - `Ok(Vec<EmbedContent<'_, '_>>)` with one or more embed contents representing messages about the activity.
	/// - `Err(anyhow::Error)` with relevant error details if a failure occurs at any step during execution.
	///
	/// # Functionality
	/// 1. Extracts interaction and context information:
	///     - Retrieves the `command_interaction` details (e.g., user input).
	///     - Gets the application context `ctx`.
	///
	/// 2. Gets bot-specific configuration and cache:
	///     - Loads `BotData` from the context.
	///     - Extracts `config` for database and localization settings.
	///
	/// 3. Parses user input:
	///     - Retrieves the `anime_name` from the subcommand map or defaults to an empty string.
	///
	/// 4. Fetches anime details from Anilist:
	///     - Queries `get_minimal_anime_media` for the anime's metadata using the cache and input name.
	///
	/// 5. Handles localization for the response:
	///     - Loads language/localization configurations for customizing messages.
	///
	/// 6. Checks for existing activities:
	///     - Verifies whether the anime is already being tracked for a specific guild using `check_if_activity_exist`.
	///
	/// 7. Prepares the response for existing activities:
	///     - Constructs an embed content response indicating the activity already exists with localized messages.
	///
	/// 8. Processes new activities:
	///     - Collects additional user-provided data like delay.
	///     - Prepares trimmed versions of anime names to fit API constraints.
	///
	/// 9. Handles image processing:
	///     - Fetches the anime's cover image from its metadata.
	///     - Resizes and converts the image to Base64 for usage in webhooks.
	///
	/// 10. Handles scheduling for next airing episodes:
	///     - Extracts and computes the timestamp for the next episode using Anilist's data.
	///
	/// 11. Configures webhook notifications:
	///     - Creates or retrieves a webhook for the designated channel.
	///
	/// 12. Inserts activity data into the database:
	///     - Persists new anime activity details into a database table via an active model (`ActivityData`).
	///
	/// 13. Returns success:
	///     - Constructs localized success message with an embed that includes information about the newly created activity.
	///
	/// # Parameters
	/// - `self`: Reference to the implementing struct.
	///
	/// # Asynchronous Behavior
	/// - Because this function performs multiple asynchronous tasks (e.g., HTTP requests, database interactions),
	///   it must be used inside an async runtime.
	///
	/// # Possible Errors
	/// - Fails if there is an issue with:
	///   - Fetching anime metadata from Anilist.
	///   - Image processing or resizing.
	///   - Interactions with webhook configuration.
	///   - Database connection or insertion.
	///   - Parsing or formatting data during execution.
	///
	/// # Example Usage
	/// ```rust
	/// let embed_contents = your_struct_instance.get_contents().await;
	/// match embed_contents {
	///     Ok(contents) => {
	///         for content in contents {
	///             println!("EmbedContent: {:?}", content);
	///         }
	///     }
	///     Err(err) => {
	///         eprintln!("Error occurred: {}", err);
	///     }
	/// }
	/// ```
	async fn get_contents(&self) -> Result<Vec<EmbedContent<'_, '_>>> {
		let command_interaction = self.get_command_interaction();
		let ctx = self.get_ctx();

		let bot_data = ctx.data::<BotData>().clone();
		let config = bot_data.config.clone();

		let map = get_option_map_string_subcommand_group(&command_interaction);
		let anime = map
			.get(&String::from("anime_name"))
			.cloned()
			.unwrap_or(String::new());

		let anilist_cache = bot_data.anilist_cache.clone();
		let media = get_minimal_anime_media(anime.to_string(), anilist_cache).await?;

		let guild_id = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("1"),
		};

		let add_activity_localised =
			load_localization_add_activity(guild_id.clone(), config.db.clone());

		self.defer().await?;

		let anime_id = media.id;
		let url = format!("https://anilist.co/anime/{}", anime_id);
		let exist = check_if_activity_exist(anime_id, guild_id.clone(), config.db.clone()).await;

		let title = media
			.title
			.ok_or(anyhow!("No title for the media".to_string()))?;
		let anime_name = get_name(title);

		if exist {
			let add_activity_localised = add_activity_localised.await?;
			let embed_content = EmbedContent::new(add_activity_localised.fail.clone())
				.description(
					add_activity_localised
						.fail_desc
						.replace("$anime$", anime_name.as_str()),
				)
				.url(Some(url))
				.command_type(EmbedType::Followup);

			return Ok(vec![embed_content]);
		}

		let channel_id = command_interaction.channel_id;

		let delay = map
			.get(&String::from("delay"))
			.unwrap_or(&String::from("0"))
			.parse()
			.unwrap_or(0);

		let trimmed_anime_name = if anime_name.len() >= 50 {
			trim_webhook(anime_name.clone(), 50 - anime_name.len() as i32)
		} else {
			anime_name.clone()
		};

		let image_url = media.cover_image.ok_or(
			anyhow!("No cover image for this media".to_string()),
		)?.extra_large.
			unwrap_or(
				"https://imgs.search.brave.com/CYnhSvdQcm9aZe3wG84YY0B19zT2wlAuAkiAGu0mcLc/rs:fit:640:400:1/g:ce/aHR0cDovL3d3dy5m/cmVtb250Z3VyZHdh/cmEub3JnL3dwLWNv/bnRlbnQvdXBsb2Fk/cy8yMDIwLzA2L25v/LWltYWdlLWljb24t/Mi5wbmc"
					.to_string()
		);
		let bytes = get(image_url.clone()).await?.bytes().await?;
		let buf = resize_image(&bytes).await?;
		let base64 = STANDARD.encode(buf.into_inner());
		let image = format!("data:image/jpeg;base64,{}", base64);

		let next_airing = media.next_airing_episode.clone().ok_or(anyhow!(format!(
			"No next episode found for {} on anilist",
			anime_name
		)))?;
		let timestamp = next_airing.airing_at as i64;
		let chrono = chrono::DateTime::<Utc>::from_timestamp(timestamp, 0)
			.unwrap_or_default()
			.naive_utc();

		let webhook = get_webhook(
			&ctx,
			channel_id,
			image.clone(),
			base64.clone(),
			trimmed_anime_name.clone(),
		)
		.await?;

		let connection = bot_data.db_connection.clone();
		ActivityData::insert(activity_data::ActiveModel {
			anime_id: Set(media.id),
			timestamp: Set(chrono),
			server_id: Set(guild_id),
			webhook: Set(webhook),
			episode: Set(next_airing.episode),
			name: Set(trimmed_anime_name),
			delay: Set(delay),
			image: Set(image.clone()),
		})
		.exec(&*connection)
		.await?;

		let add_activity_localised = add_activity_localised.await?;

		let embed_content = EmbedContent::new(add_activity_localised.success.clone())
			.description(
				add_activity_localised
					.success_desc
					.replace("$anime$", anime_name.as_str()),
			)
			.url(Some(url))
			.command_type(EmbedType::Followup);

		Ok(vec![embed_content])
	}
}

/// Asynchronously resizes an input image to a 128x128 pixel JPEG format while maintaining its aspect ratio.
///
/// This function first detects the image format, then calculates cropping parameters to create a square image.
/// After cropping, the image is resized to 128x128 pixels using a nearest-neighbor filter.
///
/// # Arguments
/// * `image_bytes` - A reference to a `Bytes` object containing the raw image data.
///
/// # Returns
/// * Returns a `Result` containing a `Cursor<Vec<u8>>` with the resized image bytes or an error if the operation fails.
///
/// # Errors
/// * Returns an error if:
///   - The image format cannot be determined.
///   - Loading or decoding the image fails.
///   - Cropping or resizing cannot be performed.
///   - Writing the resized image to the buffer fails.
///
/// # Dependencies
/// * This function uses the `image` crate for handling image processing tasks such as decoding and resizing.
/// * The `Bytes` type is used for storing the raw input image data.
/// * `Cursor` is used to store the in-memory output image result.
///
/// # Example
/// ```rust
/// use bytes::Bytes;
/// use tokio;
///
/// #[tokio::main]
/// async fn main() {
///     let image_bytes = Bytes::from(include_bytes!("test_image.jpg") as &[u8]);
///     
///     match resize_image(&image_bytes).await {
///         Ok(resized_image) => {
///             // Use the resized image (resized_image.into_inner() returns the Vec<u8> containing the image bytes)
///             println!("Image resized successfully!");
///         }
///         Err(e) => {
///             eprintln!("Failed to resize image: {}", e);
///         }
///     }
/// }
/// ```
async fn resize_image(image_bytes: &Bytes) -> Result<Cursor<Vec<u8>>> {
	let image = image::load_from_memory_with_format(image_bytes, guess_format(image_bytes)?)?;

	let (width, height) = image.dimensions();

	let (crop_x, crop_y, square_size) = calculate_crop_params(width, height);

	let resized_image = image
		.crop_imm(crop_x, crop_y, square_size, square_size)
		.resize_exact(128, 128, FilterType::Nearest);

	let mut buffer = Cursor::new(Vec::new());

	resized_image.write_to(&mut buffer, ImageFormat::Jpeg)?;

	Ok(buffer)
}

/// Calculates the cropping parameters required to extract a square crop
/// from a rectangular image defined by its width and height.
///
/// The function determines the largest square that fits within the given
/// dimensions and centers the square within the rectangle. It returns
/// the coordinates of the top-left corner of the square and its size.
///
/// # Parameters
/// - `width`: The width of the rectangular image.
/// - `height`: The height of the rectangular image.
///
/// # Returns
/// A tuple of three `u32` values:
/// - `crop_x`: The x-coordinate of the top-left corner of the square crop.
/// - `crop_y`: The y-coordinate of the top-left corner of the square crop.
/// - `square_size`: The size (width and height) of the square crop.
///
/// # Examples
/// ```
/// let width = 800;
/// let height = 600;
/// let (crop_x, crop_y, square_size) = calculate_crop_params(width, height);
/// assert_eq!(crop_x, 100);
/// assert_eq!(crop_y, 0);
/// assert_eq!(square_size, 600);
/// ```
///
/// If the rectangle is already square, the function returns `(0, 0, square_size)`:
/// ```
/// let width = 500;
/// let height = 500;
/// let (crop_x, crop_y, square_size) = calculate_crop_params(width, height);
/// assert_eq!(crop_x, 0);
/// assert_eq!(crop_y, 0);
/// assert_eq!(square_size, 500);
/// ```
fn calculate_crop_params(width: u32, height: u32) -> (u32, u32, u32) {
	let square_size = width.min(height);

	let crop_x = (width - square_size) / 2;

	let crop_y = (height - square_size) / 2;

	(crop_x, crop_y, square_size)
}

/// Asynchronously checks whether a specific activity exists in the database.
///
/// This function takes an Anime ID, a server ID, and a database configuration,
/// then connects to the database and queries for an activity that matches the given
/// Anime ID and server ID. If the activity exists, the function returns `true`,
/// otherwise it returns `false`.
///
/// # Parameters
/// - `anime_id` (`i32`): The ID of the anime to check for in the activity.
/// - `server_id` (`String`): The ID of the server associated with the activity.
/// - `config` (`DbConfig`): The database configuration required to connect to the database.
///
/// # Returns
/// - `bool`: `true` if the activity exists, `false` otherwise.
///
/// # Errors
/// - If the function fails to establish a connection to the database or encounters any
///   error while executing the query, it will return `false`.
///
/// # Examples
/// ```
/// use your_crate::your_module::{check_if_activity_exist, DbConfig};
///
/// #[tokio::main]
/// async fn main() {
///     let anime_id = 123;
///     let server_id = "server_456".to_string();
///     let config = DbConfig::new("database_url_here");
///
///     let exists = check_if_activity_exist(anime_id, server_id, config).await;
///     println!("Activity exists: {}", exists);
/// }
/// ```
///
/// # Note
/// This function uses the `sea-orm` library for interacting with the database,
/// and assumes the presence of the `ActivityData` entity and `Column` for filtering.
/// Ensure that the database schema and configurations align with these assumptions.
async fn check_if_activity_exist(anime_id: i32, server_id: String, config: DbConfig) -> bool {
	let conn = match sea_orm::Database::connect(get_url(config.clone())).await {
		Ok(conn) => conn,
		Err(_) => return false,
	};

	let row = match ActivityData::find()
		.filter(Column::ServerId.eq(server_id))
		.filter(Column::AnimeId.eq(anime_id))
		.one(&conn)
		.await
	{
		Ok(row) => row,
		Err(_) => return false,
	};

	trace!(?row);

	row.is_some()
}

/// Retrieves the name of a media title by combining its English and Romaji representations,
/// or returning whichever is available.
///
/// This function attempts to format the provided `MediaTitle` into a human-readable name.
/// - If both English and Romaji titles are available, it returns them combined in the format `"English / Romaji"`.
/// - If only one of the titles is available, it returns that title as a string.
/// - If neither title is available, it returns an empty string.
///
/// Additionally, the resolved title is logged for debugging purposes at the trace level.
///
/// # Arguments
///
/// * `title` - A `MediaTitle` containing the English and Romaji title options.
///
/// # Returns
///
/// A `String` representing the formatted media title or an empty string if neither title is available.
///
/// # Examples
///
///```rust
/// let title = MediaTitle {
///     english: Some("My Hero Academia".to_string()),
///     romaji: Some("Boku no Hero Academia".to_string()),
/// };
///
/// let result = get_name(title);
/// assert_eq!(result, "My Hero Academia / Boku no Hero Academia");
///
/// let title = MediaTitle {
///     english: Some("Attack on Titan".to_string()),
///     romaji: None,
/// };
///
/// let result = get_name(title);
/// assert_eq!(result, "Attack on Titan");
///
/// let title = MediaTitle {
///     english: None,
///     romaji: Some("Shingeki no Kyojin".to_string()),
/// };
///
/// let result = get_name(title);
/// assert_eq!(result, "Shingeki no Kyojin");
///
/// let title = MediaTitle {
///     english: None,
///     romaji: None,
/// };
///
/// let result = get_name(title);
/// assert_eq!(result, "");
/// ```
///
/// # Notes
///
/// Be sure to configure your logger to capture trace-level logs if you wish to view the debug output.
pub fn get_name(title: MediaTitle) -> String {
	let english_title = title.english;

	let romaji_title = title.romaji;

	let title = match (romaji_title, english_title) {
		(Some(romaji), Some(english)) => format!("{} / {}", english, romaji),
		(Some(romaji), None) => romaji,
		(None, Some(english)) => english,
		(None, None) => String::new(),
	};

	trace!(?title);

	title
}

/// Asynchronously retrieves or creates a Discord webhook for a given channel, sets its properties based on the provided inputs,
/// and updates the webhook's avatar with a decoded base64 image.
///
/// # Arguments
///
/// * `ctx` - A reference to the current Serenity context. Used for making API calls to Discord.
/// * `channel_id` - A generic identifier for the target channel where the webhook should be fetched or created.
/// * `image` - A URL or string representing the avatar image for the webhook.
/// * `base64` - A base64-encoded string representing the new avatar image for the webhook.
/// * `anime_name` - The desired name to be used for the webhook.
///
/// # Returns
///
/// This function returns a `Result`:
/// * `Ok(String)` - The URL of the webhook.
/// * `Err` - If an error occurs during any of the HTTP requests, JSON parsing, or decoding operations.
///
/// # Workflow
/// 1. Logs the `image` and `anime_name` values for debugging.
/// 2. Creates a webhook JSON object to be sent to Discord's API.
/// 3. Fetches the bot's current application ID for comparison with existing webhooks.
/// 4. Retrieves all webhooks for the specified channel:
///    - If no webhook exists for the bot, a new webhook is created.
///    - If a webhook exists for the bot, its URL is captured.
/// 5. Decodes the base64 avatar string into bytes.
/// 6. Updates the webhook's name and avatar with the provided `anime_name` and the decoded image.
///
/// # Errors
///
/// Possible sources of errors:
/// - Failure to fetch the current application info or channel webhooks via Discord API.
/// - If no existing webhook's user is found for comparison.
/// - Base64 decoding errors or failure to update the webhook's avatar.
///
/// # Example
/// ```rust
/// let webhook_url = get_webhook(
///     &ctx,
///     channel_id,
///     "https://example.com/avatar.jpg".to_string(),
///     base64_image_string,
///     "AnimeName".to_string(),
/// ).await?;
/// println!("Webhook URL: {}", webhook_url);
/// ```
async fn get_webhook(
	ctx: &SerenityContext, channel_id: GenericChannelId, image: String, base64: String,
	anime_name: String,
) -> Result<String> {
	trace!(?image);

	trace!(?anime_name);

	let webhook_info = json!({
		"avatar": image,
		"name": anime_name
	});

	let bot_id = ctx
		.http
		.get_current_application_info()
		.await?
		.id
		.to_string();

	trace!(?bot_id);

	let mut webhook_url = String::new();

	let webhooks = ctx
		.http
		.get_channel_webhooks(ChannelId::new(channel_id.get()))
		.await?;

	if webhooks.is_empty() {
		let webhook = ctx
			.http
			.create_webhook(ChannelId::new(channel_id.get()), &webhook_info, None)
			.await?;

		webhook_url = webhook.url()?;
	} else {
		for webhook in webhooks {
			if webhook
				.user
				.clone()
				.ok_or(anyhow!("webhook user not found"))?
				.id
				.to_string() == bot_id
			{
				webhook_url = webhook.url()?;

				break;
			}
		}

		if webhook_url.is_empty() {
			let webhook = ctx
				.http
				.create_webhook(ChannelId::new(channel_id.get()), &webhook_info, None)
				.await?;

			webhook_url = webhook.url()?;
		}
	}

	trace!(?webhook_url);

	let cursor = Cursor::new(base64);

	let mut decoder = DecoderReader::new(cursor, &STANDARD);

	let mut decoded_bytes = Vec::new();

	decoder.read_to_end(&mut decoded_bytes)?;

	let mut webhook = ctx.http.get_webhook_from_url(webhook_url.as_str()).await?;

	let attachment = CreateAttachment::bytes(decoded_bytes, "avatar");
	let attachment = attachment.encode().await?;
	let edit_webhook = EditWebhook::new().name(anime_name).avatar(attachment);

	webhook.edit(&ctx.http, edit_webhook).await?;

	Ok(webhook_url)
}

/// Asynchronously fetches minimal anime information by anime ID.
///
/// This function interacts with the AniList GraphQL API to retrieve the basic
/// data for a specific anime, using its ID. The function also utilizes a cache
/// for improved efficiency by avoiding redundant API calls.
///
/// # Arguments
///
/// * `id` - An `i32` representing the ID of the anime to fetch.
/// * `cache` - An `Arc<RwLock<Cache<String, String>>>` used to store and retrieve
///             cached responses for API queries.
///
/// # Returns
///
/// A `Result<Media>` which, on success, contains the minimal details of the
/// requested anime encapsulated in a `Media` object. On failure, it returns an error.
///
/// # Errors
///
/// This function will return an error if:
/// - There's an issue with making the request to the AniList API.
/// - The API response doesn't contain the expected `data` field.
/// - The `media` information in the response is absent.
///
/// # Examples
///
/// ```
/// use std::sync::Arc;
/// use tokio::sync::RwLock;
/// use some_cache_library::Cache;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let cache = Arc::new(RwLock::new(Cache::new()));
///     let anime_id = 12345;
///     
///     match get_minimal_anime_by_id(anime_id, cache).await {
///         Ok(media) => println!("Anime retrieved: {:?}", media),
///         Err(e) => eprintln!("Failed to fetch anime: {}", e),
///     }
///
///     Ok(())
/// }
/// ```
///
/// # Debugging
///
/// For logging purposes, the anime `id` will be traced to assist in identifying
/// any potential issues during the request execution.
pub async fn get_minimal_anime_by_id(
	id: i32, cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<Media> {
	trace!(?id);

	let query = MinimalAnimeIdVariables { id: Some(id) };

	let operation = MinimalAnimeId::build(query);

	let response: GraphQlResponse<MinimalAnimeId> =
		make_request_anilist(operation, false, cache).await?;

	let media = response
		.data
		.ok_or(anyhow!("Error with request"))?
		.media
		.ok_or(anyhow!("No media found"))?;

	Ok(media)
}

/// Fetches the minimal anime information by performing a search query.
///
/// This asynchronous function searches for a specific anime by the provided `query` string. It uses
/// a GraphQL operation defined by the `MinimalAnimeSearch` structure and attempts to return the
/// corresponding `Media` object containing the minimal information about the anime.
///
/// # Arguments
///
/// * `query` - A reference to a string representing the anime search query.
/// * `cache` - An `Arc` wrapped in a `RwLock` that holds a cache object for storing/retrieving data to optimize performance.
///
/// # Returns
///
/// This function returns a `Result`:
/// * `Ok(Media)` on successful retrieval of the anime information.
/// * `Err` if the request fails, or the desired media cannot be found.
///
/// # Errors
///
/// This function returns an error in the following cases:
/// * If the HTTP request to AniList fails or contains invalid data.
/// * If the response from AniList does not include a valid `data` field.
/// * If no media entry is found in the returned data.
///
/// # Examples
///
/// ```rust
/// use std::sync::Arc;
/// use tokio::sync::RwLock;
/// use anyhow::Result;
/// use my_crate::cache::Cache;
/// use my_crate::anime::get_minimal_anime_by_search;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let cache = Arc::new(RwLock::new(Cache::new()));
///     let query = "Naruto";
///
///     match get_minimal_anime_by_search(query, cache).await {
///         Ok(media) => println!("Found media: {:?}", media),
///         Err(e) => eprintln!("Error: {}", e),
///     }
///
///     Ok(())
/// }
/// ```
async fn get_minimal_anime_by_search(
	query: &str, cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<Media> {
	trace!(?query);

	let search_query = MinimalAnimeSearchVariables {
		search: Some(query),
	};

	let operation = MinimalAnimeSearch::build(search_query);

	let response: GraphQlResponse<MinimalAnimeSearch> =
		make_request_anilist(operation, false, cache).await?;

	let media = response
		.data
		.ok_or(anyhow!("Error with request"))?
		.media
		.ok_or(anyhow!("No media found"))?;

	Ok(media)
}

/// Retrieves minimal anime media details based on the provided anime identifier (ID or name).
///
/// This asynchronous function determines whether the input `anime` is an integer (ID) or a string
/// (name) and retrieves the relevant minimal media details accordingly. It makes use of a cache
/// to optimize repeated requests.
///
/// ## Parameters
/// - `anime`: A `String` containing either an anime ID (if it can be parsed to an `i32`)
///   or an anime name.
/// - `cache`: An `Arc<RwLock<Cache<String, String>>>` instance used for caching request results
///   to improve performance and reduce redundant API calls.
///
/// ## Returns
/// - `Ok(Media)`: The minimal media details of the anime, encapsulated in a `Media` struct.
/// - `Err(_)`: If there is an error (e.g., parsing, cache access, or retrieval failure),
///   an error is returned.
///
/// ## Example
/// ```rust
/// use tokio::sync::RwLock;
/// use std::sync::Arc;
/// use some_crate::{get_minimal_anime_media, Cache, Media};
///
/// #[tokio::main]
/// async fn main() {
///     let cache = Arc::new(RwLock::new(Cache::new()));
///     let anime_name = "Naruto".to_string();
///
///     match get_minimal_anime_media(anime_name, cache).await {
///         Ok(media) => println!("Retrieved media: {:?}", media),
///         Err(e) => eprintln!("Failed to retrieve media: {}", e),
///     }
/// }
/// ```
///
/// ## Notes
/// - Ensure the cache is properly initialized and shared among asynchronous tasks.
/// - The function uses the `get_minimal_anime_by_id` and `get_minimal_anime_by_search`
///   helper functions to handle the retrieval process depending on whether the input
///   is an ID or name.
///
/// ## Logging
/// - Debug logs (`trace!`) are emitted with the retrieved `media` details for debugging purposes.
pub async fn get_minimal_anime_media(
	anime: String, cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<Media> {
	let media = if let Ok(id) = anime.parse::<i32>() {
		get_minimal_anime_by_id(id, cache).await?
	} else {
		get_minimal_anime_by_search(&anime, cache).await?
	};

	trace!(?media);

	Ok(media)
}

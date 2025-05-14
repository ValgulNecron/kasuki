//! The `SeiyuuCommand` struct serves as the implementation of a specific Discord bot command
//! that retrieves and displays information about a seiyuu (Japanese voice actor) from AniList.
//! This struct complies with the `Command` trait.
//!
//! # Fields
//! - `ctx`: The Serenity context, used to interact with and manage the Discord bot's state and data.
//! - `command_interaction`: Represents the specific command interaction that triggered the command.
use anyhow::{Result, anyhow};
use bytes::Bytes;
use std::io::Cursor;

use crate::command::command::{Command, CommandRun, EmbedContent, EmbedImage, EmbedType};
use crate::event_handler::BotData;
use crate::helper::get_option::command::get_option_map_string;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::message::anilist_user::seiyuu::load_localization_seiyuu;
use crate::structure::run::anilist::seiyuu_id::{
	Character, CharacterConnection, SeiyuuId, SeiyuuIdVariables, Staff, StaffImage,
};
use crate::structure::run::anilist::seiyuu_search::{SeiyuuSearch, SeiyuuSearchVariables};
use cynic::{GraphQlResponse, QueryBuilder};
use image::imageops::FilterType;
use image::{DynamicImage, GenericImage, GenericImageView, ImageFormat};
use serenity::all::{CommandInteraction, Context as SerenityContext, CreateAttachment};
use small_fixed_array::FixedString;
use uuid::Uuid;

/// A structure representing a command to handle interactions related to seiyuu (voice actors) using the Serenity library.
///
/// # Fields
///
/// * `ctx` - The context of the Serenity framework containing the Discord bot's state, session, and cache data.
/// It provides access to utilities and contextual information required for handling commands or events.
///
/// * `command_interaction` - The interaction instance representing a specific command
/// issued by a user, containing details about the command and the interaction state.
///
/// # Usage
/// This struct is typically used to encapsulate the necessary context and interaction
/// details for processing a command related to seiyuu. It can contain additional methods
/// or logic to handle the command's functionality effectively.
pub struct SeiyuuCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for SeiyuuCommand {
	/// Retrieves a reference to the `SerenityContext` associated with the current instance.
	///
	/// This function provides read-only access to the `SerenityContext` stored within
	/// the instance of the struct. The `SerenityContext` is essential for interacting
	/// with Discord through the Serenity library, enabling actions such as sending messages,
	/// managing channels, and more.
	///
	/// # Returns
	///
	/// A reference to the `SerenityContext` (`&SerenityContext`).
	///
	/// # Example
	///
	/// ```rust
	/// let context = instance.get_ctx();
	/// // Use `context` for further operations
	/// ```
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	/// Retrieves a reference to the `CommandInteraction` associated with the current instance.
	///
	/// # Returns
	/// A reference to the `CommandInteraction` object stored within the instance.
	///
	/// # Example
	/// ```rust
	/// let command_interaction = instance.get_command_interaction();
	/// // Use command_interaction as needed
	/// ```
	///
	/// # Notes
	/// This function provides a read-only reference to the underlying `CommandInteraction`.
	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}

	/// Retrieves embed contents for a Discord interaction, performing operations to fetch, process,
	/// and format data related to staff and their associated characters from a remote API.
	///
	/// ### Functionality
	/// - Parses interaction input to extract the staff name or ID.
	/// - Communicates with the Anilist API to retrieve detailed staff information, including their characters.
	/// - Processes the fetched data into images and combined layouts for graphical representation.
	/// - Creates and returns a structured embed message with the processed data.
	///
	/// ### Considerations
	/// - Staff can be searched by name or ID. Adjusts the query and parsing based on input.
	/// - Handles cases where the input does not match any staff, returning relevant error messages.
	/// - Utilizes caching mechanisms to optimize data fetching and prevent redundant API calls.
	/// - Dynamically creates combined images of the staff and their associated characters for better visualization.
	///
	/// ### Returns
	/// - `Vec<EmbedContent<'_, '_>>`: A vector containing embedded content with images and metadata.
	/// - Returns an `Err` if any step encounters an issue (e.g., missing data, API call failure).
	///
	/// ### Errors
	/// - `No staff name specified`: When the input lacks a valid staff name or ID.
	/// - `No staff found`: When the search query yields no results.
	/// - `No image found`: When no image is available in the retrieved staff data.
	/// - Other errors depend on network requests, data parsing, or image processing failures.
	///
	/// ### Dependencies
	/// - External API: Makes requests to Anilist API for staff and character details.
	/// - Libraries: Uses image processing (`image` crate) and web requests (`reqwest` crate).
	///
	/// ### Example Usage
	/// ```rust
	/// let embed_contents = self.get_contents().await?;
	/// ```
	async fn get_contents(&self) -> Result<Vec<EmbedContent<'_, '_>>> {
		let ctx = self.get_ctx();
		let bot_data = ctx.data::<BotData>().clone();
		let command_interaction = self.get_command_interaction();

		let config = bot_data.config.clone();
		let anilist_cache = bot_data.anilist_cache.clone();

		let map = get_option_map_string(command_interaction);

		let value = map
			.get(&FixedString::from_str_trunc("staff_name"))
			.ok_or(anyhow!("No staff name specified"))?;

		let total_per_row = 4u32;

		let per_page = (total_per_row * total_per_row) as i32;

		let staff: Staff = if value.parse::<i32>().is_ok() {
			let id = value.parse::<i32>().unwrap();

			let var = SeiyuuIdVariables {
				id: Some(id),
				per_page: Some(per_page),
			};

			let operation = SeiyuuId::build(var);

			let data: GraphQlResponse<SeiyuuId> =
				make_request_anilist(operation, false, anilist_cache).await?;

			data.data.unwrap().page.unwrap().staff.unwrap()[0]
				.clone()
				.unwrap()
		} else {
			let var = SeiyuuSearchVariables {
				per_page: Some(per_page),
				search: Some(value),
			};

			let operation = SeiyuuSearch::build(var);

			let data: GraphQlResponse<SeiyuuSearch> =
				make_request_anilist(operation, false, anilist_cache).await?;

			let data = match data.data {
				Some(data) => match data.page {
					Some(page) => match page.staff {
						Some(staff) => match staff[0].clone() {
							Some(staff) => staff,
							None => return Err(anyhow!("No staff found")),
						},
						None => return Err(anyhow!("No staff list found")),
					},
					None => return Err(anyhow!("No page found")),
				},
				None => return Err(anyhow!("No data found")),
			};

			Staff::from(data)
		};

		let guild_id = match command_interaction.guild_id {
			Some(id) => id.to_string(),
			None => String::from("0"),
		};

		let seiyuu_localised = load_localization_seiyuu(guild_id, config.db.clone()).await?;

		self.defer().await?;

		let mut buffers: Vec<Bytes> = Vec::new();

		let staff_image = match staff.image {
			Some(image) => image,
			None => return Err(anyhow!("No image found")),
		};

		let url = get_staff_image(staff_image);

		let response = reqwest::get(url).await?;

		let bytes = response.bytes().await?;

		buffers.push(bytes);

		let character = staff.characters.unwrap();

		let characters_images_url = get_characters_image(character);

		for character_image in characters_images_url {
			let response = reqwest::get(match &character_image {
				Some(char) => match char.clone().image {
					Some(image) => match image.large {
						Some(large) => large,
						None => continue,
					},
					None => continue,
				},
				None => continue,
			})
			.await?;

			let bytes = match response.bytes().await {
				Ok(bytes) => bytes,
				Err(_) => continue,
			};

			buffers.push(bytes);
		}

		let mut images: Vec<DynamicImage> = Vec::new();

		for bytes in &buffers {
			// Load the image from the byte vector
			images.push(image::load_from_memory(bytes)?);
		}

		let (width, height) = images[0].dimensions();

		let sub_image = images[0].to_owned().crop(0, 0, width, height);

		let aspect_ratio = width as f32 / height as f32;

		let new_height = 1000 * total_per_row;

		let new_width = (new_height as f32 * aspect_ratio) as u32;

		let smaller_height = new_height / total_per_row;

		let smaller_width = new_width / total_per_row;

		let total_width = smaller_width * total_per_row + new_width;

		let mut combined_image = DynamicImage::new_rgba16(total_width, new_height);

		let resized_img =
			image::imageops::resize(&sub_image, new_width, new_height, FilterType::CatmullRom);

		combined_image.copy_from(&resized_img, 0, 0).unwrap();

		let mut pos_list = Vec::new();

		for x in 0..total_per_row {
			for y in 0..total_per_row {
				pos_list.push((new_width + (smaller_width * y), smaller_height * x))
			}
		}

		images.remove(0);

		for (i, img) in images.iter().enumerate() {
			let (width, height) = img.dimensions();

			let sub_image = img.to_owned().crop(0, 0, width, height);

			let resized_img = image::imageops::resize(
				&sub_image,
				smaller_width,
				smaller_height,
				FilterType::CatmullRom,
			);

			let (pos_width, pos_height) = pos_list[i];

			combined_image.copy_from(&resized_img, pos_width, pos_height)?;
		}

		let combined_uuid = Uuid::new_v4();

		let image_path = &format!("{}.png", combined_uuid);

		let rgba8_image = combined_image.to_rgba8();

		let mut bytes: Vec<u8> = Vec::new();

		rgba8_image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::WebP)?;

		let attachment = CreateAttachment::bytes(bytes, image_path.to_string());

		let image = EmbedImage {
			attachment,
			image: image_path.clone(),
		};

		let embed_content = EmbedContent::new(seiyuu_localised.title)
			.command_type(EmbedType::Followup)
			.images(Some(vec![image]));

		Ok(vec![embed_content])
	}
}

/// Retrieves a list of characters from a `CharacterConnection`.
///
/// This function takes a `CharacterConnection` as input and extracts
/// the `nodes` field, which is an `Option` containing a vector of
/// characters. It then unwraps the `Option` and returns the vector of
/// `Option<Character>`. If the `nodes` field is `None`, this function
/// will panic due to the use of `unwrap()`.
///
/// # Arguments
///
/// * `character` - A `CharacterConnection` object that holds a
///   connection to character nodes.
///
/// # Returns
///
/// A `Vec<Option<Character>>` containing the character nodes from the
/// given `CharacterConnection`.
///
/// # Panics
///
/// This function will panic if the `nodes` field of the `CharacterConnection`
/// is `None`. To prevent panics, ensure that the `nodes` field is always
/// populated before calling this function.
///
/// # Example
///
/// ```rust
/// let character_connection = CharacterConnection {
///     nodes: Some(vec![Some(Character::new("Hero")), None]),
/// };
/// let characters = get_characters_image(character_connection);
/// assert_eq!(characters.len(), 2);
/// assert!(characters[0].is_some());
/// assert!(characters[1].is_none());
/// ```

fn get_characters_image(character: CharacterConnection) -> Vec<Option<Character>> {
	character.nodes.unwrap()
}

/// Retrieves the URL of the large staff image.
///
/// # Parameters
/// - `staff`: A `StaffImage` instance containing optional image fields.
///
/// # Returns
/// A `String` representing the URL of the large staff image.
///
/// # Panics
/// This function will panic if the `large` field within the `StaffImage` instance is `None`.
/// Ensure that the `large` field is `Some` before calling this function.
///
/// # Example
/// ```
/// let staff_image = StaffImage { large: Some(String::from("example.com/large_image.jpg")) };
/// let image_url = get_staff_image(staff_image);
/// assert_eq!(image_url, "example.com/large_image.jpg");
/// ```

fn get_staff_image(staff: StaffImage) -> String {
	staff.large.unwrap()
}

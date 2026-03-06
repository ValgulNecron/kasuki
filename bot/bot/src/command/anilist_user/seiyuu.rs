//! The `SeiyuuCommand` struct serves as the implementation of a specific Discord bot command
//! that retrieves and displays information about a seiyuu (Japanese voice actor) from AniList.
//! This struct complies with the `Command` trait.
//!
//! # Fields
//! - `ctx`: The Serenity context, used to interact with and manage the Discord bot's state and data.
//! - `command_interaction`: Represents the specific command interaction that triggered the command.
use anyhow::anyhow;
use bytes::Bytes;
use std::io::Cursor;

use crate::command::command::CommandRun;
use crate::command::context::CommandContext;
use crate::command::embed_content::{CommandFiles, EmbedContent, EmbedsContents};
use crate::helper::get_option::command::get_option_map_string;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::run::anilist::seiyuu_id::{
	Character, CharacterConnection, SeiyuuId, SeiyuuIdVariables, Staff, StaffImage,
};
use crate::structure::run::anilist::seiyuu_search::{SeiyuuSearch, SeiyuuSearchVariables};
use cynic::{GraphQlResponse, QueryBuilder};
use fluent_templates::Loader;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImage, GenericImageView, ImageFormat};
use kasuki_macros::slash_command;
use serenity::all::{CommandInteraction, Context as SerenityContext};
use shared::localization::USABLE_LOCALES;
use small_fixed_array::FixedString;
use uuid::Uuid;

#[slash_command(
	name = "seiyuu", desc = "Info of a seiyuu.", command_type = ChatInput,
	contexts = [Guild, BotDm, PrivateChannel],
	install_contexts = [Guild, User],
	args = [(name = "staff_name", desc = "Name of the seiyuu you want to check.", arg_type = String, required = true, autocomplete = true)],
)]
async fn seiyuu_command(self_: SeiyuuCommand) -> Result<EmbedsContents<'_>> {
	let cx = CommandContext::new(
		self_.get_ctx().clone(),
		self_.get_command_interaction().clone(),
	);
	let anilist_cache = cx.anilist_cache.clone();

	let map = get_option_map_string(&cx.command_interaction);

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
			make_request_anilist(operation, true, anilist_cache).await?;

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
			make_request_anilist(operation, true, anilist_cache).await?;

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

	let lang_id = cx.lang_id().await;


	let mut buffers: Vec<Bytes> = Vec::new();

	let staff_image = match staff.image {
		Some(image) => image,
		None => return Err(anyhow!("No image found")),
	};

	let url = get_staff_image(staff_image);

	let client = &cx.bot_data.http_client;
	let response = client.get(url).send().await?;

	let bytes = response.bytes().await?;

	buffers.push(bytes);

	let character = staff.characters.unwrap();

	let characters_images_url = get_characters_image(character);

	for character_image in characters_images_url {
		let response = client
			.get(match &character_image {
				Some(char) => match char.clone().image {
					Some(image) => match image.large {
						Some(large) => large,
						None => continue,
					},
					None => continue,
				},
				None => continue,
			})
			.send()
			.await?;

		let bytes = match response.bytes().await {
			Ok(bytes) => bytes,
			Err(_) => continue,
		};

		buffers.push(bytes);
	}

	// Move all CPU-heavy image work to a blocking thread
	let (image_path, bytes) =
		tokio::task::spawn_blocking(move || -> anyhow::Result<(String, Vec<u8>)> {
			let mut images: Vec<DynamicImage> = Vec::new();

			for buf in &buffers {
				images.push(image::load_from_memory(buf)?);
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

			let path = format!("{}.png", combined_uuid);

			let rgba8_image = combined_image.to_rgba8();

			let mut bytes: Vec<u8> = Vec::new();

			rgba8_image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::WebP)?;

			Ok((path, bytes))
		})
		.await
		.map_err(|e| anyhow!("spawn_blocking panicked: {}", e))??;

	let image_path = &image_path;

	let image_url = format!("attachment://{}", image_path.clone());
	let image = CommandFiles::new(image_path.clone(), bytes);

	let embed_content =
		EmbedContent::new(USABLE_LOCALES.lookup(&lang_id, "anilist_user_seiyuu-title"))
			.images_url(image_url);

	let embed_contents = EmbedsContents::new(vec![embed_content])
		.add_files(vec![image])
		.clone();

	Ok(embed_contents)
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

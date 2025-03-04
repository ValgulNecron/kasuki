use anyhow::{Result, anyhow};
use std::io::Cursor;

use crate::command::command_trait::{
	Command, Embed, EmbedContent, EmbedImage, EmbedType, SlashCommand,
};
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
use prost::bytes::Bytes;
use serenity::all::{CommandInteraction, Context as SerenityContext, CreateAttachment};
use small_fixed_array::FixedString;
use uuid::Uuid;

pub struct SeiyuuCommand {
	pub ctx: SerenityContext,
	pub command_interaction: CommandInteraction,
}

impl Command for SeiyuuCommand {
	fn get_ctx(&self) -> &SerenityContext {
		&self.ctx
	}

	fn get_command_interaction(&self) -> &CommandInteraction {
		&self.command_interaction
	}
}

impl SlashCommand for SeiyuuCommand {
	async fn run_slash(&self) -> Result<()> {
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

		let content = EmbedContent {
			title: seiyuu_localised.title,
			description: "".to_string(),
			thumbnail: None,
			url: None,
			command_type: EmbedType::Followup,
			colour: None,
			fields: vec![],
			images: Some(vec![image]),
			action_row: None,
			images_url: None,
		};

		self.send_embed(content).await
	}
}

/// Retrieves the characters associated with a staff member from the AniList API.
///
/// This function takes a `StaffImageWrapper` object, which contains data about a staff member fetched from the AniList API,
/// and returns a vector of `StaffImageNodes` objects, each of which represents a character associated with the staff member.
///
/// # Arguments
///
/// * `staff` - A `StaffImageWrapper` object containing data about a staff member.
///
/// # Returns
///
/// A `Vec<StaffImageNodes>` that contains the characters associated with the staff member.

fn get_characters_image(character: CharacterConnection) -> Vec<Option<Character>> {
	character.nodes.unwrap()
}

/// Retrieves the image of a staff member from the AniList API.
///
/// This function takes a `StaffImageWrapper` object, which contains data about a staff member fetched from the AniList API,
/// and returns a string that represents the URL of the staff member's image.
///
/// # Arguments
///
/// * `staff` - A `StaffImageWrapper` object containing data about a staff member.
///
/// # Returns
///
/// A `String` that represents the URL of the staff member's image.

fn get_staff_image(staff: StaffImage) -> String {
	staff.large.unwrap()
}

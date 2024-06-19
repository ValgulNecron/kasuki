use std::io::Cursor;

use cynic::{GraphQlResponse, QueryBuilder};
use image::imageops::FilterType;
use image::{DynamicImage, GenericImage, GenericImageView, ImageFormat};
use prost::bytes::Bytes;
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandInteraction, Context, CreateAttachment, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage,
};
use uuid::Uuid;

use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::message::anilist_user::seiyuu::load_localization_seiyuu;
use crate::structure::run::anilist::seiyuu_id::{
    Character, CharacterConnection, SeiyuuId, SeiyuuIdVariables, Staff, StaffImage,
};
use crate::structure::run::anilist::seiyuu_search::{SeiyuuSearch, SeiyuuSearchVariables};

/// Executes the command to fetch and display information about a seiyuu (voice actor) from AniList.
///
/// This function retrieves the name or ID of the seiyuu from the command interaction and fetches the seiyuu's data from AniList.
/// It then creates a combined image of the seiyuu and the characters they have voiced, and sends this image as a response to the command interaction.
/// The function also handles errors that may occur during the execution of the command, such as errors in fetching data from AniList, creating the image, or sending the response.
///
/// # Arguments
///
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command executed successfully, or `Err` if an error occurred.
pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    let map = get_option_map_string_subcommand(command_interaction);
    let value = map.get(&String::from("staff_name")).ok_or(AppError::new(
        String::from("There is no option"),
        ErrorType::Option,
        ErrorResponseType::Message,
    ))?;
    let total_per_row = 4u32;
    let per_page = (total_per_row * total_per_row) as i32;
    let staff: Staff = if value.parse::<i32>().is_ok() {
        let id = value.parse::<i32>().unwrap();
        let var = SeiyuuIdVariables {
            id: Some(id),
            per_page: Some(per_page),
        };
        let operation = SeiyuuId::build(var);
        let data: GraphQlResponse<SeiyuuId> = make_request_anilist(operation, false).await?;

        data.data.unwrap().page.unwrap().staff.unwrap()[0]
            .clone()
            .unwrap()
    } else {
        let var = SeiyuuSearchVariables {
            per_page: Some(per_page),
            search: Some(value),
        };
        let operation = SeiyuuSearch::build(var);
        let data: GraphQlResponse<SeiyuuSearch> = make_request_anilist(operation, false).await?;
        let data = data.data.unwrap().page.unwrap().staff.unwrap()[0]
            .clone()
            .unwrap();
        Staff::from(data)
    };

    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let seiyuu_localised = load_localization_seiyuu(guild_id).await?;

    let builder_message = Defer(CreateInteractionResponseMessage::new());

    command_interaction
        .create_response(&ctx.http, builder_message)
        .await
        .map_err(|e| {
            AppError::new(
                format!("Error while sending the command {}", e),
                ErrorType::Command,
                ErrorResponseType::Message,
            )
        })?;
    let mut buffers: Vec<Bytes> = Vec::new();
    let staff_image = staff.image.unwrap();
    let url = get_staff_image(staff_image);
    let response = reqwest::get(url).await.map_err(|e| {
        AppError::new(
            format!("Error while getting the response from the server. {}", e),
            ErrorType::WebRequest,
            ErrorResponseType::Followup,
        )
    })?;
    let bytes = response.bytes().await.map_err(|e| {
        AppError::new(
            format!("Failed to get bytes data from response. {}", e),
            ErrorType::WebRequest,
            ErrorResponseType::Followup,
        )
    })?;
    buffers.push(bytes);
    let character = staff.characters.unwrap();
    let characters_images_url = get_characters_image(character);
    for character_image in characters_images_url {
        let response = reqwest::get(&character_image.unwrap().image.unwrap().large.unwrap())
            .await
            .map_err(|e| {
                AppError::new(
                    format!("Error while getting the response from the server. {}", e),
                    ErrorType::WebRequest,
                    ErrorResponseType::Followup,
                )
            })?;

        let bytes = response.bytes().await.map_err(|e| {
            AppError::new(
                format!("Failed to get bytes data from response. {}", e),
                ErrorType::WebRequest,
                ErrorResponseType::Followup,
            )
        })?;
        buffers.push(bytes);
    }

    let mut images: Vec<DynamicImage> = Vec::new();
    for bytes in &buffers {
        // Load the image from the byte vector
        images.push(image::load_from_memory(bytes).map_err(|e| {
            AppError::new(
                format!("Failed to create the image from the file. {}", e),
                ErrorType::Image,
                ErrorResponseType::Followup,
            )
        })?);
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
        combined_image
            .copy_from(&resized_img, pos_width, pos_height)
            .map_err(|e| {
                AppError::new(
                    format!("Failed to copy the image. {}", e),
                    ErrorType::Image,
                    ErrorResponseType::Followup,
                )
            })?;
    }

    let combined_uuid = Uuid::new_v4();
    let image_path = &format!("{}.png", combined_uuid);

    let builder_embed = get_default_embed(None)
        .image(format!("attachment://{}", &image_path))
        .title(&seiyuu_localised.title);

    let rgba8_image = combined_image.to_rgba8();
    let mut bytes: Vec<u8> = Vec::new();
    rgba8_image
        .write_to(&mut Cursor::new(&mut bytes), ImageFormat::WebP)
        .map_err(|e| {
            AppError::new(
                format!("Failed to write the image to the buffer. {}", e),
                ErrorType::Image,
                ErrorResponseType::Followup,
            )
        })?;
    let attachment = CreateAttachment::bytes(bytes, image_path.to_string());

    let builder_message = CreateInteractionResponseFollowup::new()
        .embed(builder_embed)
        .files(vec![attachment]);

    command_interaction
        .create_followup(&ctx.http, builder_message)
        .await
        .map_err(|e| {
            AppError::new(
                format!("Error while sending the command {}", e),
                ErrorType::Command,
                ErrorResponseType::Followup,
            )
        })?;
    Ok(())
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

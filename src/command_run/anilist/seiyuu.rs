use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use image::imageops::FilterType;
use image::{DynamicImage, GenericImage, GenericImageView};
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandDataOption, CommandInteraction, Context, CreateAttachment, CreateEmbed,
    CreateInteractionResponseFollowup, CreateInteractionResponseMessage, Timestamp,
};
use tracing::{debug, error};
use uuid::Uuid;

use crate::anilist_struct::run::seiyuu::{StaffImageNodes, StaffImageWrapper};
use crate::command_run::get_option::get_option_map_string;
use crate::common::get_option_value::get_option;
use crate::constant::COLOR;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::lang_struct::anilist::seiyuu::load_localization_seiyuu;

pub async fn run(
    ctx: &Context,
    command_interaction: &CommandInteraction,
) -> Result<(), AppError> {
    let map = get_option_map_string(command_interaction);
    let value = map
        .get(&String::from("seiyuu_name"))
        .ok_or(
            AppError::new(
                String::from("There is no option"),
                ErrorType::Option,
                ErrorResponseType::Message,
            )
        )?;

    let data = if value.parse::<i32>().is_ok() {
        StaffImageWrapper::new_staff_by_id(value.parse().unwrap()).await?
    } else {
        StaffImageWrapper::new_staff_by_search(&value).await?
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
    let mut uuids: Vec<Uuid> = Vec::new();
    for _ in 0..5 {
        let uuid = Uuid::new_v4();
        uuids.push(uuid)
    }

    let url = get_staff_image(data.clone());
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
    let mut buffer = File::create(format!("{}.png", uuids[0])).map_err(|e| {
        AppError::new(
            format!("Failed to write the file bytes. {}", e),
            ErrorType::File,
            ErrorResponseType::Followup,
        )
    })?;
    buffer.write_all(&bytes).map_err(|e| {
        AppError::new(
            format!("Failed to write the file bytes. {}", e),
            ErrorType::File,
            ErrorResponseType::Followup,
        )
    })?;
    let mut i = 1;
    let characters_images_url = get_characters_image(data.clone());
    for character_image in characters_images_url {
        let response = reqwest::get(&character_image.image.large)
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
        let mut buffer = File::create(format!("{}.png", uuids[i])).map_err(|e| {
            AppError::new(
                format!("Failed to write the file bytes. {}", e),
                ErrorType::File,
                ErrorResponseType::Followup,
            )
        })?;
        buffer.write_all(&bytes).map_err(|e| {
            AppError::new(
                format!("Failed to write the file bytes. {}", e),
                ErrorType::File,
                ErrorResponseType::Followup,
            )
        })?;
        i += 1
    }

    let mut images: Vec<DynamicImage> = Vec::new();
    for uuid in &uuids {
        let path = format!("{}.png", uuid);
        let img_path = Path::new(&path);
        // Read the image file into a byte vector
        let mut file = match File::open(img_path) {
            Ok(f) => f,
            Err(e) => {
                error!("{}", e);
                continue;
            }
        };

        let mut buffer = Vec::new();
        match file.read_to_end(&mut buffer) {
            Ok(f) => f,
            Err(e) => {
                error!("{}", e);
                continue;
            }
        };

        // Load the image from the byte vector
        images.push(image::load_from_memory(&buffer).map_err(|e| {
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
    let new_height = 2000;
    let new_width = (new_height as f32 * aspect_ratio) as u32;

    let smaller_height = new_height / 2;
    let smaller_width = new_width / 2;

    let total_width = smaller_width * 2 + new_width;

    let mut combined_image = DynamicImage::new_rgba16(total_width, 2000);

    let resized_img =
        image::imageops::resize(&sub_image, new_width, new_height, FilterType::CatmullRom);
    combined_image.copy_from(&resized_img, 0, 0).unwrap();
    let pos_list = [
        (new_width, 0),
        (new_width + smaller_width, 0),
        (new_width, smaller_height),
        (new_width + smaller_width, smaller_height),
    ];
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
    combined_image
        .save(format!("{}.png", combined_uuid))
        .map_err(|e| {
            AppError::new(
                format!("Failed to write the file bytes. {}", e),
                ErrorType::File,
                ErrorResponseType::Followup,
            )
        })?;
    uuids.push(combined_uuid);
    let image_path = &format!("{}.png", combined_uuid);

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .image(format!("attachment://{}", &image_path))
        .title(&seiyuu_localised.title);

    let attachment = CreateAttachment::path(&image_path).await.map_err(|e| {
        AppError::new(
            format!("Error while uploading the attachment {}", e),
            ErrorType::Command,
            ErrorResponseType::Followup,
        )
    })?;

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

    for uuid in uuids {
        let path = format!("{}.png", uuid);
        match fs::remove_file(path) {
            Ok(_) => debug!("File {} has been removed successfully", uuid),
            Err(e) => error!("Failed to remove file {}: {}", uuid, e),
        }
    }
    Ok(())
}

fn get_characters_image(staff: StaffImageWrapper) -> Vec<StaffImageNodes> {
    staff.data.staff.characters.nodes
}

fn get_staff_image(staff: StaffImageWrapper) -> String {
    staff.data.staff.image.large
}

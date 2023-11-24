use crate::constant::COLOR;
use crate::function::error_management::common::{custom_error, custom_followup_error};
use crate::function::error_management::error_response::error_writing_file_response_edit;
use crate::function::general::differed_response::differed_response;
use crate::function::general::in_progress::in_progress_embed;
use crate::structure::anilist::staff::struct_staff_image::StaffImageWrapper;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImage, GenericImageView};
use log::{debug, error};
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::command::CommandOptionType;
use serenity::model::prelude::application_command::{
    ApplicationCommandInteraction, CommandDataOption, CommandDataOptionValue,
};
use serenity::model::Timestamp;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use uuid::Uuid;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    let option = options
        .get(0)
        .expect("Expected name option")
        .resolved
        .as_ref()
        .expect("Expected name object");

    if let CommandDataOptionValue::String(value) = option {
        let data = if value.parse::<i32>().is_ok() {
            match StaffImageWrapper::new_staff_by_id(value.parse().unwrap()).await {
                Ok(staff_wrapper) => staff_wrapper,
                Err(error) => {
                    custom_error(ctx, command, &error).await;
                    return;
                }
            }
        } else {
            match StaffImageWrapper::new_staff_by_search(value).await {
                Ok(staff_wrapper) => staff_wrapper,
                Err(error) => {
                    custom_error(ctx, command, &error).await;
                    return;
                }
            }
        };

        differed_response(ctx, command).await;

        let mut message = match in_progress_embed(ctx, command).await {
            Ok(message) => match message {
                Some(real_message) => real_message,
                None => {
                    error!("There where a big error.");
                    return;
                }
            },
            Err(e) => {
                error!("{}", e);
                custom_followup_error(ctx, command, e.as_str()).await;
                return;
            }
        };

        let mut uuids: Vec<Uuid> = Vec::new();
        for _ in 0..5 {
            let uuid = Uuid::new_v4();
            uuids.push(uuid);
        }

        let url = data.get_staff_image();
        let response = match reqwest::get(url).await {
            Ok(resp) => resp,
            Err(e) => {
                error!("{}", e);
                return;
            }
        };
        let bytes = match response.bytes().await {
            Ok(b) => b.as_ref().to_owned(),
            Err(e) => {
                error!("{}", e);
                return;
            }
        };

        let mut buffer = match File::create(format!("{}.png", uuids[0])) {
            Ok(buff) => buff,
            Err(e) => {
                error_writing_file_response_edit(ctx, command, message.clone()).await;
                error!("{}", e);
                return;
            }
        };

        match buffer.write_all(&bytes) {
            Ok(buff) => buff,
            Err(e) => {
                error_writing_file_response_edit(ctx, command, message.clone()).await;
                error!("{}", e);
                return;
            }
        }

        let mut i = 1;
        let characters_images_url = data.get_characters_image();
        for character_image in characters_images_url {
            let response = match reqwest::get(&character_image.image.large).await {
                Ok(resp) => resp,
                Err(e) => {
                    error!("{}", e);
                    return;
                }
            };
            let bytes = match response.bytes().await {
                Ok(b) => b.as_ref().to_owned(),
                Err(e) => {
                    error!("{}", e);
                    return;
                }
            };

            let mut buffer = match File::create(format!("{}.png", uuids[i])) {
                Ok(buff) => buff,
                Err(e) => {
                    error_writing_file_response_edit(ctx, command, message.clone()).await;
                    error!("{}", e);
                    return;
                }
            };

            match buffer.write_all(&bytes) {
                Ok(buff) => buff,
                Err(e) => {
                    error_writing_file_response_edit(ctx, command, message.clone()).await;
                    error!("{}", e);
                    return;
                }
            }
            i += 1
        }

        let mut images = Vec::new();
        for uuid in &uuids {
            let path = format!("{}.png", uuid);
            let img_path = Path::new(&path);

            // Read the image file into a byte vector
            let mut file = match File::open(img_path) {
                Ok(file) => file,
                Err(e) => {
                    error!("Failed to open the file: {}", e);
                    continue;
                }
            };

            let mut buffer = Vec::new();
            match file.read_to_end(&mut buffer) {
                Ok(_) => (),
                Err(e) => {
                    error!("Failed to read the file: {}", e);
                    continue;
                }
            };

            // Load the image from the byte vector
            match image::load_from_memory(&buffer) {
                Ok(img) => images.push(img),
                Err(e) => {
                    error!("Failed to load the image: {}", e);
                    continue;
                }
            };
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
                .unwrap();
        }

        let combined_uuid = Uuid::new_v4();
        combined_image
            .save(format!("{}.png", combined_uuid))
            .unwrap();
        uuids.push(combined_uuid);
        let image_path = &format!("{}.png", combined_uuid);
        let combined_image_path = Path::new(image_path);

        if let Err(why) = message
            .edit(&ctx.http, |m| {
                m.attachment(combined_image_path).embed(|e| {
                    e.title("seiyuu")
                        .image(format!("attachment://{}", image_path))
                        .timestamp(Timestamp::now())
                        .color(COLOR)
                })
            })
            .await
        {
            error!("Error creating slash command: {}", why);
        }

        for uuid in uuids {
            let path = format!("{}.png", uuid);
            match fs::remove_file(path) {
                Ok(_) => debug!("File {} has been removed successfully", uuid),
                Err(e) => error!("Failed to remove file {}: {}", uuid, e),
            }
        }
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("seiyuu")
        .description("Get an image of a seiyuu with 4 of the role.")
        .create_option(|option| {
            let option = option
                .name("seiyuu_name")
                .description("Name of the seiyuu.")
                .kind(CommandOptionType::String)
                .required(true)
                .set_autocomplete(true);
            option
        })
}

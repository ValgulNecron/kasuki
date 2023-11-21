use crate::constant::COLOR;
use crate::function::error_management::common::{custom_error, custom_followup_error};
use crate::function::error_management::error_response::error_writing_file_response_edit;
use crate::function::general::differed_response::differed_response;
use crate::function::general::in_progress::in_progress_embed;
use crate::structure::anilist::staff::struct_staff_image::StaffImageWrapper;
use image::{open, DynamicImage, GenericImage, GenericImageView};
use log::error;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::application::command::CommandOptionType;
use serenity::model::prelude::application_command::{
    ApplicationCommandInteraction, CommandDataOption, CommandDataOptionValue,
};
use serenity::model::Timestamp;
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

        let mut images: Vec<Vec<u8>> = Vec::new();
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

        let mut buffer = match File::create(uuids[0].to_string()) {
            Ok(buff) => buff,
            Err(e) => {
                error_writing_file_response_edit(ctx, command, message.clone()).await;
                error!("{}", e);
                return;
            }
        };

        match buffer.write_all(&*bytes) {
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

            let mut buffer = match File::create(uuids[i].to_string()) {
                Ok(buff) => buff,
                Err(e) => {
                    error_writing_file_response_edit(ctx, command, message.clone()).await;
                    error!("{}", e);
                    return;
                }
            };

            match buffer.write_all(&*bytes) {
                Ok(buff) => buff,
                Err(e) => {
                    error_writing_file_response_edit(ctx, command, message.clone()).await;
                    error!("{}", e);
                    return;
                }
            }
            i = i + 1
        }

        let mut images = Vec::new();
        for uuid in uuids {
            let path = uuid.to_string();
            let img_path = Path::new(&path);

            // Open the image file
            match open(&img_path) {
                Ok(img) => images.push(img),
                Err(e) => {
                    error!("Failed to open the image: {}", e);
                    return;
                }
            }
        }

        let mut combined_image = DynamicImage::new_luma8(600, 600);

        for (i, img) in images.iter().enumerate() {
            let (width, height) = img.dimensions();
            let mut sub_image = img.to_owned().crop(0, 0, width, height);
            combined_image
                .copy_from(&sub_image, 0, i as u32 * width)
                .unwrap();
        }
        combined_image.save("combined_image.png").unwrap();

        if let Err(why) = message
            .edit(&ctx.http, |m| {
                m.embed(|e| e.title("seiyuu").timestamp(Timestamp::now()).color(COLOR))
            })
            .await
        {
            println!("Error creating slash command: {}", why);
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

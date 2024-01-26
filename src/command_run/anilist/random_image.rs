use crate::constant::{COLOR, COMMAND_SENDING_ERROR, DIFFERED_COMMAND_SENDING_ERROR};
use crate::error_enum::AppError;
use crate::error_enum::AppError::{
    DifferedFailedToGetBytes, DifferedResponseError, DifferedWritingFile,
};
use crate::lang_struct::anilist::random_image::{
    load_localization_random_image, RandomImageLocalised,
};
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandDataOption, CommandInteraction, Context, CreateAttachment, CreateEmbed,
    CreateInteractionResponseFollowup, CreateInteractionResponseMessage, Timestamp,
};
use std::fs;
use uuid::Uuid;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command_interaction: &CommandInteraction,
) -> Result<(), AppError> {
    let mut image_type = String::new();

    for option in options {
        if option.name.as_str() == "image_type" {
            image_type = match option.value.as_str() {
                Some(image_type) => image_type.to_string(),
                None => String::from("megumin"),
            };
        }
    }
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };
    let random_image_localised = load_localization_random_image(guild_id).await?;

    let builder_message = Defer(CreateInteractionResponseMessage::new());

    command_interaction
        .create_response(&ctx.http, builder_message)
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())?;

    send_embed(ctx, command_interaction, image_type, random_image_localised).await
}

async fn send_embed(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    image_type: String,
    random_image_localised: RandomImageLocalised,
) -> Result<(), AppError> {
    let url = format!("https://api.waifu.pics/sfw/{}", image_type);
    let resp = reqwest::get(&url).await.map_err(|_| {
        DifferedResponseError(String::from("Failed to get the response from the server."))
    })?;
    let json: serde_json::Value = resp.json().await.map_err(|_| {
        DifferedResponseError(String::from("Failed to get the response from the server."))
    })?;
    let image_url = json["url"]
        .as_str()
        .ok_or("No image found")
        .map_err(|_| DifferedResponseError(String::from("Failed to get data from url.")))?
        .to_string();

    let response = reqwest::get(image_url)
        .await
        .map_err(|_| DifferedResponseError(String::from("Failed to get data from url.")))?;
    let bytes = response.bytes().await.map_err(|_| {
        DifferedFailedToGetBytes(String::from("Failed to get bytes data from response."))
    })?;

    let uuid_name = Uuid::new_v4();
    let filename = format!("{}.gif", uuid_name);
    let filename_str = filename.as_str();

    fs::write(&filename, &bytes)
        .map_err(|_| DifferedWritingFile(String::from("Failed to write the file bytes.")))?;

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .image(format!("attachment://{}", &filename))
        .title(random_image_localised.title);

    let attachement = CreateAttachment::path(&filename)
        .await
        .map_err(|_| DIFFERED_COMMAND_SENDING_ERROR.clone())?;

    let builder_message = CreateInteractionResponseFollowup::new()
        .embed(builder_embed)
        .files(vec![attachement]);

    command_interaction
        .create_followup(&ctx.http, builder_message)
        .await
        .map_err(|_| DIFFERED_COMMAND_SENDING_ERROR.clone())?;

    let _ = fs::remove_file(filename_str);

    Ok(())
}

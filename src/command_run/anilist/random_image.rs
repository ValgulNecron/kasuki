use serenity::all::{
    CommandInteraction, Context, CreateAttachment, CreateEmbed, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage, Timestamp,
};
use serenity::all::CreateInteractionResponse::Defer;
use uuid::Uuid;

use crate::command_run::get_option::get_option_map_string;
use crate::constant::COLOR;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::lang_struct::anilist::random_image::load_localization_random_image;

pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    let map = get_option_map_string(command_interaction);
    let image_type = map.get(&String::from("image_type")).ok_or(AppError::new(
        String::from("There is no option"),
        ErrorType::Option,
        ErrorResponseType::Followup,
    ))?;

    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };
    let random_image_localised = load_localization_random_image(guild_id).await?;

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
    send_embed(ctx, command_interaction, image_type, random_image_localised.title, "sfw").await
}

pub async fn send_embed(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    image_type: &String,
    title: String,
    endpoint: &str,
) -> Result<(), AppError> {
    let url = format!("https://api.waifu.pics/{}/{}", endpoint, image_type);
    let resp = reqwest::get(&url).await.map_err(|e| {
        AppError::new(
            format!("Error while getting the response from the server. {}", e),
            ErrorType::WebRequest,
            ErrorResponseType::Followup,
        )
    })?;
    let json: serde_json::Value = resp.json().await.map_err(|e| {
        AppError::new(
            format!("Failed to get the json from the server response. {}", e),
            ErrorType::WebRequest,
            ErrorResponseType::Followup,
        )
    })?;
    let image_url = json["url"]
        .as_str()
        .ok_or("No image found")
        .map_err(|e| {
            AppError::new(
                format!("Failed to get data from url. {}", e),
                ErrorType::WebRequest,
                ErrorResponseType::Followup,
            )
        })?
        .to_string();

    let response = reqwest::get(image_url).await.map_err(|e| {
        AppError::new(
            format!("Failed to get data from url. {}", e),
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

    let uuid_name = Uuid::new_v4();
    let filename = format!("{}.gif", uuid_name);

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .image(format!("attachment://{}", &filename))
        .title(title);

    let attachment = CreateAttachment::bytes(bytes, &filename);

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

use std::env;

use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde_json::{json, Value};
use serenity::all::{
    CommandInteraction, Context, CreateAttachment, CreateEmbed, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage, Timestamp,
};
use serenity::all::CreateInteractionResponse::Defer;
use tracing::{info, trace};
use uuid::Uuid;

use crate::command_run::get_option::get_option_map_string;
use crate::constant::{COLOR, DEFAULT_STRING, IMAGE_BASE_URL, IMAGE_MODELS, IMAGE_TOKEN};
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::image_saver::general_image_saver::image_saver;
use crate::lang_struct::ai::image::load_localization_image;

pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let map = get_option_map_string(command_interaction);
    let prompt = map
        .get(&String::from("description"))
        .unwrap_or(DEFAULT_STRING);

    let image_localised = load_localization_image(guild_id.clone()).await?;

    let builder_message = Defer(CreateInteractionResponseMessage::new());

    command_interaction
        .create_response(&ctx.http, builder_message)
        .await
        .map_err(|e| {
            AppError::new(
                format!("Error while sending the command {}", e),
                ErrorType::Option,
                ErrorResponseType::Message,
            )
        })?;

    let uuid_name = Uuid::new_v4();
    let filename = format!("{}.png", uuid_name);

    let model = unsafe { IMAGE_MODELS.as_str() };
    info!("{}", model);

    let quality = match env::var("AI_IMAGE_QUALITY") {
        Ok(quality) => Some(quality),
        Err(_) => None,
    };
    let style = match env::var("AI_IMAGE_STYLE") {
        Ok(style) => Some(style),
        Err(_) => None,
    };

    let size = env::var("AI_IMAGE_SIZE").unwrap_or(String::from("1024x1024"));

    let data: Value = match (quality, style) {
        (Some(quality), Some(style)) => {
            json!({
                "prompt": prompt,
                "n": 1,
                "size": size,
                "model": model,
                "quality": quality,
                "style": style,
                "response_format": "url"
            })
        }
        (None, Some(style)) => {
            json!({
                "prompt": prompt,
                "n": 1,
                "size": size,
                "model": model,
                "style": style,
                "response_format": "url"
            })
        }
        (Some(quality), None) => {
            json!({
                "prompt": prompt,
                "n": 1,
                "size": size,
                "model": model,
                "quality": quality,
                "response_format": "url"
            })
        }
        (None, None) => {
            json!({
                "prompt": prompt,
                "n": 1,
                "size": size,
                "model": model,
                "response_format": "url"
            })
        }
    };

    trace!("{:#?}", data);

    let client = reqwest::Client::new();

    let token = unsafe { IMAGE_TOKEN.as_str() };
    trace!("{}", token);
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        match HeaderValue::from_str(&format!("Bearer {}", token)) {
            Ok(data) => data,
            Err(e) => {
                return Err(AppError::new(
                    format!("Failed to create the header. {}", e),
                    ErrorType::WebRequest,
                    ErrorResponseType::Followup,
                ));
            }
        },
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    trace!("{:#?}", data);
    let url = unsafe { IMAGE_BASE_URL.as_str() };
    trace!("{}", url);
    let res: Value = client
        .post(url)
        .headers(headers)
        .json(&data)
        .send()
        .await
        .map_err(|e| {
            AppError::new(
                format!("Failed to get the response from the server. {}", e),
                ErrorType::WebRequest,
                ErrorResponseType::Followup,
            )
        })?
        .json()
        .await
        .map_err(|e| {
            AppError::new(
                format!("Failed to get the response from the server. {}", e),
                ErrorType::WebRequest,
                ErrorResponseType::Followup,
            )
        })?;
    trace!("{:#?}", res);

    let url_string = res
        .get("data")
        .ok_or(AppError::new(
            String::from("Failed to get data from result"),
            ErrorType::Option,
            ErrorResponseType::Followup,
        ))?
        .get(0)
        .ok_or(AppError::new(
            String::from("Failed to get the first image"),
            ErrorType::Option,
            ErrorResponseType::Followup,
        ))?
        .get("url")
        .ok_or(AppError::new(
            String::from("Failed to get the url from the result"),
            ErrorType::Option,
            ErrorResponseType::Followup,
        ))?
        .as_str()
        .ok_or(AppError::new(
            String::from("Failed to convert to str."),
            ErrorType::Option,
            ErrorResponseType::Followup,
        ))?;

    let response = reqwest::get(url_string).await.map_err(|e| {
        AppError::new(
            format!("Failed to get the response from the server. {}", e),
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

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .image(format!("attachment://{}", &filename))
        .title(image_localised.title);

    let attachment = CreateAttachment::bytes(bytes.clone(), &filename);

    let builder_message = CreateInteractionResponseFollowup::new()
        .embed(builder_embed)
        .files(vec![attachment]);

    command_interaction
        .create_followup(&ctx.http, builder_message)
        .await
        .map_err(|e| {
            AppError::new(
                format!("Error while sending the command {}", e),
                ErrorType::Option,
                ErrorResponseType::Followup,
            )
        })?;

    image_saver(guild_id, filename.clone(), bytes.to_vec()).await?;

    Ok(())
}

use std::{env, fs};

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::{json, Value};
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandDataOption, CommandDataOptionValue, CommandInteraction, Context, CreateAttachment,
    CreateEmbed, CreateInteractionResponseFollowup, CreateInteractionResponseMessage, Timestamp,
};
use tracing::trace;
use uuid::Uuid;

use crate::constant::COLOR;
use crate::error_enum::AppError;
use crate::error_enum::AppError::{DifferedError, Error};
use crate::error_enum::DifferedError::{DifferedCommandSendingError, DifferedOptionError, FailedToGetBytes, FailedUrlError, HeaderError, ImageModelError, ResponseError, TokenError, WritingFile};
use crate::error_enum::Error::{ErrorCommandSendingError, ErrorOptionError};
use crate::image_saver::general_image_saver::image_saver;
use crate::lang_struct::ai::image::load_localization_image;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command_interaction: &CommandInteraction,
) -> Result<(), AppError> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let lang = options
        .first()
        .ok_or(Error(ErrorOptionError(String::from("There is no option"))))?;
    let lang = lang.value.clone();

    let desc = match lang {
        CommandDataOptionValue::String(lang) => lang,
        _ => {
            return Err(Error(ErrorOptionError(String::from(
                "The command contain no option.",
            ))));
        }
    };

    let image_localised = load_localization_image(guild_id.clone()).await?;

    let builder_message = Defer(CreateInteractionResponseMessage::new());

    command_interaction
        .create_response(&ctx.http, builder_message)
        .await
        .map_err(|e| {
            Error(ErrorCommandSendingError(format!(
                "Error while sending the command {}",
                e
            )))
        })?;

    let uuid_name = Uuid::new_v4();
    let filename = format!("{}.png", uuid_name);
    let filename_str = filename.as_str();

    let prompt = desc;
    let api_key = match env::var("AI_IMAGE_API_TOKEN") {
        Ok(x) => x,
        Err(_) => {
            return Err(DifferedError(TokenError(String::from(
                "There was an error while getting the token.",
            ))));
        }
    };

    let api_base_url =
        env::var("AI_IMAGE_API_BASE_URL").unwrap_or("https://api.openai.com/v1/".to_string());

    let mut data = json!({
        "model": "dall-e-3",
        "prompt": prompt,
        "n": 1,
        "size": "1024x1024",
        "response_format": "url"
    });
    if let Ok(image_generation_mode) = env::var("IMAGE_GENERATION_MODELS_ON") {
        let is_ok_image = image_generation_mode.to_lowercase() == "true";
        let quality = match env::var("IMAGE_QUALITY") {
            Ok(quality) => Some(quality),
            Err(_) => None,
        };
        let style = match env::var("IMAGE_STYLE") {
            Ok(style) => Some(style),
            Err(_) => None,
        };

        let model = match env::var("IMAGE_GENERATION_MODELS") {
            Ok(data) => data,
            Err(e) => {
                return Err(DifferedError(ImageModelError(format!(
                    "Please specify the models you want to use. {}",
                    e
                ))));
            }
        };

        let size = env::var("IMAGE_SIZE").unwrap_or(String::from("1024x1024"));
        match (is_ok_image, quality, style) {
            (true, Some(quality), Some(style)) => {
                data = json!({
                    "prompt": prompt,
                    "n": 1,
                    "size": size,
                    "model": model,
                    "quality": quality,
                    "style": style,
                    "response_format": "url"
                })
            }
            (true, None, Some(style)) => {
                data = json!({
                    "prompt": prompt,
                    "n": 1,
                    "size": size,
                    "model": model,
                    "style": style,
                    "response_format": "url"
                })
            }
            (true, Some(quality), None) => {
                data = json!({
                    "prompt": prompt,
                    "n": 1,
                    "size": size,
                    "model": model,
                    "quality": quality,
                    "response_format": "url"
                })
            }
            (true, None, None) => {
                data = json!({
                    "prompt": prompt,
                    "n": 1,
                    "size": size,
                    "model": model,
                    "response_format": "url"
                })
            }
            (false, Some(quality), Some(style)) => {
                data = json!({
                    "prompt": prompt,
                    "n": 1,
                    "size": size,
                    "model": "dall-e-3",
                    "quality": quality,
                    "style": style,
                    "response_format": "url"
                })
            }
            (false, None, Some(style)) => {
                data = json!({
                    "prompt": prompt,
                    "n": 1,
                    "size": size,
                    "model": "dall-e-3",
                    "style": style,
                    "response_format": "url"
                })
            }
            (false, Some(quality), None) => {
                data = json!({
                    "prompt": prompt,
                    "n": 1,
                    "size": size,
                    "model": "dall-e-3",
                    "quality": quality,
                    "response_format": "url"
                })
            }
            (false, None, None) => {
                data = json!({
                    "prompt": prompt,
                    "n": 1,
                    "size": size,
                    "model": "dall-e-3",
                    "response_format": "url"
                })
            }
        }
    }
    trace!("{:#?}", data);

    let api_url = format!("{}images/generations", api_base_url);
    let client = reqwest::Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        match HeaderValue::from_str(&format!("Bearer {}", api_key)) {
            Ok(data) => data,
            Err(e) => {
                return Err(DifferedError(HeaderError(format!(
                    "Failed to create the header. {}",
                    e
                ))));
            }
        },
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let res: Value = client
        .post(api_url)
        .headers(headers)
        .json(&data)
        .send()
        .await
        .map_err(|e| {
            DifferedError(ResponseError(format!(
                "Failed to get the response from the server. {}",
                e
            )))
        })?
        .json()
        .await
        .map_err(|e| {
            DifferedError(ResponseError(format!(
                "Failed to get the response from the server. {}",
                e
            )))
        })?;
    trace!("{:#?}", res);

    let url_string = res
        .get("data")
        .ok_or(DifferedError(DifferedOptionError(String::from(
            "Failed to get data from result",
        ))))?
        .get(0)
        .ok_or(DifferedError(DifferedOptionError(String::from(
            "Failed to get the first image",
        ))))?
        .get("url")
        .ok_or(DifferedError(DifferedOptionError(String::from(
            "Failed to get the url from the result",
        ))))?
        .as_str()
        .ok_or(DifferedError(FailedUrlError(String::from(
            "Failed to convert to str.",
        ))))?;

    let response = reqwest::get(url_string).await.map_err(|e| {
        DifferedError(ResponseError(format!(
            "Failed to get bytes data from response. {}",
            e
        )))
    })?;
    let bytes = response.bytes().await.map_err(|e| {
        DifferedError(FailedToGetBytes(format!(
            "Failed to get bytes data from response. {}",
            e
        )))
    })?;

    fs::write(&filename, &bytes).map_err(|e| {
        DifferedError(WritingFile(format!(
            "Failed to write the file bytes.{}",
            e
        )))
    })?;

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .image(format!("attachment://{}", &filename))
        .title(image_localised.title);

    let attachment = CreateAttachment::path(&filename).await.map_err(|e| {
        DifferedError(DifferedCommandSendingError(format!(
            "Error while uploading the attachment {}",
            e
        )))
    })?;

    let builder_message = CreateInteractionResponseFollowup::new()
        .embed(builder_embed)
        .files(vec![attachment]);

    command_interaction
        .create_followup(&ctx.http, builder_message)
        .await
        .map_err(|e| {
            DifferedError(DifferedCommandSendingError(format!(
                "Error while sending the command {}",
                e
            )))
        })?;

    image_saver(
        guild_id,
        command_interaction.user.id.to_string(),
        filename.clone(),
        bytes.to_vec(),
    )
    .await?;

    let _ = fs::remove_file(filename_str);

    Ok(())
}

use std::{env, fs};

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::{json, Value};
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandDataOption, CommandDataOptionValue, CommandInteraction, Context, CreateAttachment,
    CreateEmbed, CreateInteractionResponseFollowup, CreateInteractionResponseMessage, Timestamp,
};
use uuid::Uuid;

use crate::constant::{
    COLOR, COMMAND_SENDING_ERROR, DIFFERED_COMMAND_SENDING_ERROR, DIFFERED_OPTION_ERROR,
    OPTION_ERROR,
};
use crate::error_enum::AppError;
use crate::error_enum::AppError::{
    DifferedFailedToGetBytes, DifferedFailedUrlError, DifferedHeaderError, DifferedImageModelError,
    DifferedResponseError, DifferedTokenError, DifferedWritingFile, NoCommandOption,
};
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

    let lang = options.get(0).ok_or(OPTION_ERROR.clone())?;
    let lang = lang.value.clone();

    let desc = match lang {
        CommandDataOptionValue::String(lang) => lang,
        _ => {
            return Err(NoCommandOption(String::from(
                "The command contain no option.",
            )));
        }
    };

    let image_localised = load_localization_image(guild_id).await?;

    let builder_message = Defer(CreateInteractionResponseMessage::new());

    command_interaction
        .create_response(&ctx.http, builder_message)
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())?;

    let uuid_name = Uuid::new_v4();
    let filename = format!("{}.png", uuid_name);
    let filename_str = filename.as_str();

    let prompt = desc;
    let api_key = match env::var("AI_IMAGE_API_TOKEN") {
        Ok(x) => x,
        Err(_) => {
            return Err(DifferedTokenError(String::from(
                "There was an error while getting the token.",
            )));
        }
    };

    let api_base_url = env::var("AI_IMAGE_API_BASE_URL")
        .unwrap_or_else(|_| "https://api.openai.com/v1/".to_string());

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
            Err(_) => {
                return Err(DifferedImageModelError(String::from(
                    "Please specify the models you want to use",
                )));
            }
        };

        let size = env::var("IMAGE_GENERATION_MODELS_ON").unwrap_or(String::from("1024x1024"));
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

    let api_url = format!("{}images/generations", api_base_url);
    let client = reqwest::Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        match HeaderValue::from_str(&format!("Bearer {}", api_key)) {
            Ok(data) => data,
            Err(_) => {
                return Err(DifferedHeaderError(String::from(
                    "Failed to create the header",
                )));
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
        .map_err(|_| {
            DifferedResponseError(String::from("Failed to get the response from the server."))
        })?
        .json()
        .await
        .map_err(|_| {
            DifferedResponseError(String::from("Failed to get the response from the server."))
        })?;

    let url_string = res
        .get("data")
        .ok_or(DIFFERED_OPTION_ERROR.clone())?
        .get(0)
        .ok_or(DIFFERED_OPTION_ERROR.clone())?
        .get("url")
        .ok_or(DIFFERED_OPTION_ERROR.clone())?
        .as_str()
        .ok_or(DifferedFailedUrlError(String::from(
            "Failed to get the response url.",
        )))?;

    let response = reqwest::get(url_string)
        .await
        .map_err(|_| DifferedResponseError(String::from("Failed to get data from url.")))?;
    let bytes = response.bytes().await.map_err(|_| {
        DifferedFailedToGetBytes(String::from("Failed to get bytes data from response."))
    })?;

    fs::write(&filename, &bytes)
        .map_err(|_| DifferedWritingFile(String::from("Failed to write the file bytes.")))?;

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .image(format!("attachment://{}", &filename))
        .title(image_localised.title);

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

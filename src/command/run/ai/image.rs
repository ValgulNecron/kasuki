use std::env;

use prost::bytes::Bytes;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandInteraction, Context, CreateAttachment, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage,
};
use tracing::{info, trace};
use uuid::Uuid;

use crate::constant::{DEFAULT_STRING, IMAGE_BASE_URL, IMAGE_MODELS, IMAGE_TOKEN};
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::get_option::subcommand::{
    get_option_map_integer_subcommand, get_option_map_string_subcommand,
};
use crate::structure::message::ai::image::{load_localization_image, ImageLocalised};

/// This module contains the implementation of the `run` function for handling AI image generation.
///
/// The `run` function is an asynchronous function that handles a command interaction for generating an AI image.
/// It retrieves the description for the image from the command interaction options and sends a deferred response to the command interaction.
/// It then generates an AI image based on the description using the OpenAI API and sends a followup message with the generated image.
///
/// The `run` function uses several helper functions and constants defined in other modules.
/// It uses the `get_option_map_string_subcommand_group` function to retrieve the description from the command interaction options.
/// It uses the `load_localization_image` function to load the localized language data for the guild.
/// It uses the `image_saver` function to save the generated image.
/// It uses the `IMAGE_MODELS`, `IMAGE_TOKEN`, and `IMAGE_BASE_URL` constants for the OpenAI API request.
///
/// The `run` function handles any errors that occur during the process and returns an `AppError` if an error occurs.
///
/// # Arguments
///
/// * `ctx` - The context in which this function is being called.
/// * `command_interaction` - The command interaction that triggered this function.
///
/// # Returns
///
/// A `Result` indicating whether the function executed successfully. If an error occurred, it contains an `AppError`.
pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let map = get_option_map_string_subcommand(command_interaction);
    trace!("{:#?}", map);
    let prompt = map
        .get(&String::from("description"))
        .unwrap_or(DEFAULT_STRING);

    let map = get_option_map_integer_subcommand(command_interaction);
    let n = *map.get(&String::from("n")).unwrap_or(&1);

    trace!(prompt);

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

    let model = IMAGE_MODELS;
    let model = model.as_str();
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
                "n": n,
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
                "n": n,
                "size": size,
                "model": model,
                "style": style,
                "response_format": "url"
            })
        }
        (Some(quality), None) => {
            json!({
                "prompt": prompt,
                "n": n,
                "size": size,
                "model": model,
                "quality": quality,
                "response_format": "url"
            })
        }
        (None, None) => {
            json!({
                "prompt": prompt,
                "n": n,
                "size": size,
                "model": model,
                "response_format": "url"
            })
        }
    };

    trace!("{:#?}", data);

    let client = reqwest::Client::new();

    let token = IMAGE_TOKEN;
    let token = token.as_str();
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
    let url = IMAGE_BASE_URL;
    let url = url.as_str();
    trace!(token);
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
    let bytes = get_image_from_response(res).await?;

    if n == 1 {
        image_with_n_equal_1(
            image_localised,
            filename,
            command_interaction,
            ctx,
            bytes[0].clone(),
        )
        .await?
    } else {
        image_with_n_greater_than_1(image_localised, filename, command_interaction, ctx, bytes)
            .await?
    }
    Ok(())
}

async fn image_with_n_equal_1(
    image_localised: ImageLocalised,
    filename: String,
    command_interaction: &CommandInteraction,
    ctx: &Context,
    bytes: Bytes,
) -> Result<(), AppError> {
    let builder_embed = get_default_embed(None)
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
    Ok(())
}

async fn image_with_n_greater_than_1(
    image_localised: ImageLocalised,
    filename: String,
    command_interaction: &CommandInteraction,
    ctx: &Context,
    bytes: Vec<Bytes>,
) -> Result<(), AppError> {
    let message = image_localised.title;
    let attachments: Vec<CreateAttachment> = bytes
        .iter()
        .enumerate()
        .map(|(index, byte)| {
            let filename = format!("{}_{}.png", filename, index);
            CreateAttachment::bytes(byte.clone(), filename)
        })
        .collect();
    let builder_message = CreateInteractionResponseFollowup::new()
        .content(message)
        .files(attachments);

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
    Ok(())
}

async fn get_image_from_response(json: Value) -> Result<Vec<Bytes>, AppError> {
    let mut bytes = Vec::new();
    let root: Root = match serde_json::from_value(json.clone()) {
        Ok(root) => root,
        Err(e) => {
            let root1: Result<Root1, serde_json::error::Error> = serde_json::from_value(json);
            return match root1 {
                Ok(root1) => Err(AppError::new(
                    format!("Failed to parse the json. {}", root1.error.message),
                    ErrorType::File,
                    ErrorResponseType::Followup,
                )),
                Err(e) => Err(AppError::new(
                    format!("Failed to parse the json. {}", e),
                    ErrorType::File,
                    ErrorResponseType::Followup,
                )),
            };
        }
    };
    let urls: Vec<String> = root.data.iter().map(|data| data.url.clone()).collect();
    trace!("{:?}", urls);
    for url in urls {
        let client = reqwest::Client::new();
        let res = client.get(url).send().await.map_err(|e| {
            AppError::new(
                format!("Failed to get the response from the server. {}", e),
                ErrorType::WebRequest,
                ErrorResponseType::Followup,
            )
        })?;
        let body = match res.bytes().await {
            Ok(body) => body,
            Err(e) => {
                return Err(AppError::new(
                    format!("Failed to get the body from the response. {}", e),
                    ErrorType::WebRequest,
                    ErrorResponseType::Followup,
                ));
            }
        };
        bytes.push(body);
    }
    Ok(bytes)
}

#[derive(Debug, Deserialize)]
struct Root {
    created: u64,
    #[serde(rename = "data")]
    data: Vec<Data>,
}

#[derive(Debug, Deserialize)]
struct Data {
    url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Error {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: String,
    pub param: Option<String>,
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Root1 {
    pub error: Error,
}

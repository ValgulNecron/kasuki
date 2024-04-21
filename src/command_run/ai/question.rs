use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::{json, Value};
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage, Timestamp,
};
use tracing::trace;

use crate::common::get_option::subcommand::get_option_map_string_subcommand;
use crate::constant::{CHAT_BASE_URL, CHAT_MODELS, CHAT_TOKEN, COLOR, DEFAULT_STRING};
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    let map = get_option_map_string_subcommand(command_interaction);
    let prompt = map.get(&String::from("prompt")).unwrap_or(DEFAULT_STRING);
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

    let api_key = CHAT_TOKEN.clone();
    let api_base_url = CHAT_BASE_URL.clone();
    let model = CHAT_MODELS.clone();

    let text = question(prompt, api_key, api_base_url, model).await?;

    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .description(text);

    let builder_message = CreateInteractionResponseFollowup::new().embed(builder_embed);

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

async fn question(
    text: &String,
    api_key: String,
    api_base_url: String,
    model: String,
) -> Result<String, AppError> {
    let api_url = api_base_url.to_string();
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", api_key)).unwrap(),
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let data = json!({
         "model": model,
         "messages": [{"role": "system", "content": "You are a helpful assistant."},{"role": "user", "content": text}]
    });
    trace!("{:?}", data);

    let res: Value = client
        .post(api_url)
        .headers(headers)
        .json(&data)
        .send()
        .await
        .map_err(|e| {
            AppError::new(
                format!("error during the request to the api. {}", e),
                ErrorType::WebRequest,
                ErrorResponseType::Followup,
            )
        })?
        .json()
        .await
        .map_err(|e| {
            AppError::new(
                format!("error getting the response. {}", e),
                ErrorType::WebRequest,
                ErrorResponseType::Followup,
            )
        })?;
    trace!("{:?}", res);
    let content = res["choices"][0]["message"]["content"].to_string();

    Ok(content.replace("\\n", " \n "))
}

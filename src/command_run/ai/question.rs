use crate::command_run::get_option::get_option_map_string;
use crate::constant::{CHAT_BASE_URL, CHAT_MODELS, CHAT_TOKEN, COLOR, DEFAULT_STRING};
use crate::error_management::interaction_error::InteractionError;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::{json, Value};
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage, Timestamp,
};
use tracing::trace;

pub async fn run(
    ctx: &Context,
    command_interaction: &CommandInteraction,
) -> Result<(), InteractionError> {
    let map = get_option_map_string(command_interaction);
    let prompt = map.get(&String::from("prompt")).unwrap_or(DEFAULT_STRING);
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

    let api_key = unsafe { CHAT_TOKEN.clone() };
    let api_base_url = unsafe { CHAT_BASE_URL.clone() };
    let model = unsafe { CHAT_MODELS.clone() };

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
            DifferedError(DifferedCommandSendingError(format!(
                "Error while sending the command {}",
                e
            )))
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
        .map_err(|e| DifferedError(ResponseError(format!("error during translation. {}", e))))?
        .json()
        .await
        .map_err(|e| DifferedError(ResponseError(format!("error during translation. {}", e))))?;
    trace!("{:?}", res);
    let content = res["choices"][0]["message"]["content"].to_string();

    Ok(content.replace("\\n", " \n "))
}

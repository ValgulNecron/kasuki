use std::error::Error;
use std::sync::Arc;

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::{json, Value};
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage,
};
use tracing::trace;

use crate::config::Config;
use crate::constant::DEFAULT_STRING;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::{FollowupError, ResponseError};
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;

/// This asynchronous function runs the command interaction for asking a question to the AI.
///
/// It first retrieves the prompt for the question from the command interaction options.
/// It then sends a deferred response to the command interaction.
///
/// It generates an AI response to the question using the OpenAI API and sends a followup message with the response.
///
/// # Arguments
///
/// * `ctx` - The context in which this function is being called.
/// * `command_interaction` - The command interaction that triggered this function.
///
/// # Returns
///
/// A `Result` indicating whether the function executed successfully. If an error occurred, it contains an `AppError`.
pub async fn run(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
) -> Result<(), Box<dyn Error>> {
    let map = get_option_map_string_subcommand(command_interaction);
    let prompt = map.get(&String::from("prompt")).unwrap_or(DEFAULT_STRING);
    let builder_message = Defer(CreateInteractionResponseMessage::new());

    command_interaction
        .create_response(&ctx.http, builder_message)
        .await
        .map_err(|e| ResponseError::Sending(format!("{:#?}", e)))?;

    let api_key = config
        .ai
        .question
        .ai_question_token
        .clone()
        .unwrap_or_default();
    let api_base_url = config
        .ai
        .question
        .ai_question_base_url
        .clone()
        .unwrap_or_default();
    let model = config
        .ai
        .question
        .ai_question_model
        .clone()
        .unwrap_or_default();

    let text = question(prompt, api_key, api_base_url, model).await?;

    let builder_embed = get_default_embed(None).description(text);

    let builder_message = CreateInteractionResponseFollowup::new().embed(builder_embed);

    command_interaction
        .create_followup(&ctx.http, builder_message)
        .await
        .map_err(|e| FollowupError::Option(format!("{:#?}", e)))?;

    Ok(())
}

/// This asynchronous function generates an AI response to a question.
///
/// It sends a request to the OpenAI API with the question and retrieves the AI's response.
/// It then formats the response and returns it.
///
/// # Arguments
///
/// * `text` - The question to ask the AI.
/// * `api_key` - The API key for the OpenAI API.
/// * `api_base_url` - The base URL for the OpenAI API.
/// * `model` - The model to use for the AI.
///
/// # Returns
///
/// A `Result` containing the AI's response. If an error occurred, it contains an `AppError`.
async fn question(
    text: &String,
    api_key: String,
    api_base_url: String,
    model: String,
) -> Result<String, Box<dyn Error>> {
    let api_url = api_base_url.to_string();
    // check the last 3 characters of the url if it v1/ or v1 or something else
    let api_url = question_api_url(api_url);
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
        .map_err(|e| FollowupError::WebRequest(format!("{:#?}", e)))?
        .json()
        .await
        .map_err(|e| FollowupError::Json(format!("{:#?}", e)))?;
    trace!("{:?}", res);
    let content = res["choices"][0]["message"]["content"].to_string();

    // replace the first and last " in the string
    let content = content[1..content.len() - 1].to_string();

    Ok(content.replace("\\n", " \n "))
}

pub fn question_api_url(api_url: String) -> String {
    if api_url.ends_with("v1/") {
        format!("{}chat/completions", api_url)
    } else if api_url.ends_with("v1") {
        format!("{}/chat/completions", api_url)
    } else {
        format!("{}/v1/chat/completions", api_url)
    }
}

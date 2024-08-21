use std::error::Error;
use std::sync::Arc;

use crate::command::command_trait::{
    get_user_sub, send_premium_response, Command, PremiumCommand, SlashCommand,
};
use crate::config::Config;
use crate::constant::{DEFAULT_STRING, MAX_FREE_AI_IMAGES, PAID_IMAGE_MULTIPLIER};
use crate::event_handler::Handler;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::FollowupError;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::{json, Value};
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage,
};
use tracing::trace;

pub struct QuestionCommand<'de> {
    pub ctx: Context,
    pub command_interaction: CommandInteraction,
    pub config: Arc<Config>,
    pub handler: &'de Handler,
    pub command_name: String,
}

impl Command for QuestionCommand<'_> {
    fn get_ctx(&self) -> &Context {
        &self.ctx
    }

    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for QuestionCommand<'_> {
    async fn run_slash(&self) -> Result<(), Box<dyn Error>> {
        if self
            .check_hourly_limit(self.command_name.clone(), self.handler)
            .await?
        {
            return Err(Box::new(FollowupError::Option(String::from(
                "You have reached your hourly limit. Please try again later.",
            ))));
        }
        send_embed(&self.ctx, &self.command_interaction, self.config.clone()).await
    }
}

impl PremiumCommand for QuestionCommand<'_> {
    async fn check_hourly_limit(
        &self,
        command_name: impl Into<String> + Clone,
        handler: &Handler,
    ) -> Result<bool, Box<dyn Error>> {
        if !handler.bot_data.config.bot.config.respect_premium {
            return Ok(false);
        }

        let usage = handler
            .get_hourly_usage(
                command_name.into(),
                self.command_interaction.user.id.to_string(),
            )
            .await;

        let (user_sub, available_sub) = get_user_sub(&self.ctx, &self.command_interaction).await?;
        if available_sub.is_none() {
            return Ok(false);
        }
        if usage <= MAX_FREE_AI_IMAGES as u128 && user_sub.is_none() {
            return Ok(false);
        }
        if usage <= (MAX_FREE_AI_IMAGES as f64 * PAID_IMAGE_MULTIPLIER) as u128
            && user_sub.is_some()
        {
            return Ok(false);
        }
        send_premium_response(&self.ctx, &self.command_interaction, available_sub).await?;
        Ok(true)
    }
}

async fn send_embed(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
) -> Result<(), Box<dyn Error>> {
    let map = get_option_map_string_subcommand(command_interaction);
    let prompt = map.get(&String::from("prompt")).unwrap_or(DEFAULT_STRING);
    let builder_message = Defer(CreateInteractionResponseMessage::new());

    command_interaction
        .create_response(&ctx.http, builder_message)
        .await?;

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
        .await?;

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
        HeaderValue::from_str(&format!("Bearer {}", api_key))?,
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
        .await?
        .json()
        .await?;
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

use regex::Regex;
use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseFollowup, CreateInteractionResponseMessage, Timestamp,
};
use tracing::error;

use crate::constant::COLOR;
use crate::event_handler::Handler;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Clone)]
pub enum Error {
    #[error("Error while getting guild data: {0}")]
    GettingGuild(String),
    #[error("Error while getting an option: {0}")]
    Option(String),
    #[error("Error while processing image: {0}")]
    ImageProcessing(String),
    #[error("Error while doing a web request: {0}")]
    WebRequest(String),
    #[error("Error while getting a byte: {0}")]
    Byte(String),
    #[error("Error while doing a webhook request: {0}")]
    Webhook(String),
    #[error("Error with the database: {0}")]
    Database(String),
    #[error("Error while sending the response: {0}")]
    Sending(String),
    #[error("Error while initializing the logger: {0}")]
    Logger(String),
    #[error("The channel is not nsfw but the media is.")]
    AdultMedia,
    #[error("Error with the JSON: {0}")]
    Json(String),
    #[error("Error with the audio: {0}")]
    Audio(String),
    #[error("Error with the file: {0}")]
    File(String),
}

/// ERROR_MESSAGE is a constant string that contains the default error message
const ERROR_MESSAGE: &str = "**There was an error while processing the command**\
    \n**This error is most likely an input error** \
    **like searching for non existent anime, requesting nsfw image to the ai, etc.**\n \
    **but in some case it's a server or programming error**.\
    **in this case report it to me and I will try to fix it the fastest I can**";

/// `command_dispatching` is an asynchronous function that handles the dispatching of commands.
/// It takes an `error`, `command_interaction`, and `ctx` as parameters.
/// `error` is an AppError, `command_interaction` is a reference to a CommandInteraction, and `ctx` is a reference to a Context.
/// It does not return a value.
///
/// # Arguments
///
/// * `error` - An AppError that represents the error.
/// * `command_interaction` - A reference to a CommandInteraction that represents the command interaction.
/// * `ctx` - A reference to a Context that represents the context.
pub async fn command_dispatching(
    message: String,
    command_interaction: &CommandInteraction,
    ctx: &Context,
    self_handler: &Handler,
) {
    error!("{}", message.replace("\\n", "\n"));
    match send_error(message.clone(), command_interaction, ctx, self_handler).await {
        Ok(_) => {}
        Err(_) => {
            match send_differed_error(message, command_interaction, ctx, self_handler).await {
                Ok(_) => {}
                Err(e) => {
                    error!("{}", e);
                }
            }
        }
    }
}

/// `send_error` is an asynchronous function that sends an error message.
/// It takes an `e`, `command_interaction`, and `ctx` as parameters.
/// `e` is an AppError, `command_interaction` is a reference to a CommandInteraction, and `ctx` is a reference to a Context.
/// It returns a Result which is either an empty tuple or an AppError.
///
/// # Arguments
///
/// * `e` - An AppError that represents the error.
/// * `command_interaction` - A reference to a CommandInteraction that represents the command interaction.
/// * `ctx` - A reference to a Context that represents the context.
///
/// # Returns
///
/// * `Result<(), AppError>` - A Result type which is either an empty tuple or an AppError.
async fn send_error(
    e: String,
    command_interaction: &CommandInteraction,
    ctx: &Context,
    self_handler: &Handler,
) -> Result<(), String> {
    let error_message = format!("{}\n{}", ERROR_MESSAGE, e);
    // censor url and token in the error message
    let error_message = censor_url_and_token(error_message, self_handler);
    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .description(error_message)
        .title("There was an error while processing the command");
    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);

    let builder = CreateInteractionResponse::Message(builder_message);

    command_interaction
        .create_response(&ctx.http, builder)
        .await
        .map_err(|e| format!("{:#?}", e))?;
    Ok(())
}

/// `send_differed_error` is an asynchronous function that sends a differed error message.
/// It takes an `e`, `command_interaction`, and `ctx` as parameters.
/// `e` is an AppError, `command_interaction` is a reference to a CommandInteraction, and `ctx` is a reference to a Context.
/// It returns a Result which is either an empty tuple or an AppError.
///
/// # Arguments
///
/// * `e` - An AppError that represents the error.
/// * `command_interaction` - A reference to a CommandInteraction that represents the command interaction.
/// * `ctx` - A reference to a Context that represents the context.
///
/// # Returns
///
/// * `Result<(), AppError>` - A Result type which is either an empty tuple or an AppError.
async fn send_differed_error(
    e: String,
    command_interaction: &CommandInteraction,
    ctx: &Context,
    self_handler: &Handler,
) -> Result<(), String> {
    let error_message = format!("{}\n{}", ERROR_MESSAGE, e);
    // censor url and token in the error message
    let error_message = censor_url_and_token(error_message, self_handler);
    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .description(error_message)
        .title("There was an error while processing the command");

    let builder = CreateInteractionResponseFollowup::new().embed(builder_embed);

    let _ = command_interaction
        .create_followup(&ctx.http, builder)
        .await
        .map_err(|e| format!("{:#?}", e))?;

    Ok(())
}

/// `censor_url_and_token` is a function that censors URLs and tokens in the given error message.
/// It takes an `error_message` as a parameter.
/// `error_message` is a String.
/// It returns a String which is the censored error message.
///
/// # Arguments
///
/// * `error_message` - A String that represents the error message.
///
/// # Returns
///
/// * `String` - A String which is the censored error message.
fn censor_url_and_token(error_message: String, self_handler: &Handler) -> String {
    let config = self_handler.bot_data.config.clone();
    let mut error_message = error_message;
    let discord_token = config.bot.discord_token.clone();
    let db_user = config.bot.config.user.clone().unwrap_or_default();
    let db_pass = config.bot.config.password.clone().unwrap_or_default();
    let db_port = config
        .bot
        .config
        .port
        .clone()
        .unwrap_or_default()
        .to_string();
    let db_host = config.bot.config.host.clone().unwrap_or_default();
    let image_token = config.ai.image.ai_image_token.clone().unwrap_or_default();
    let transcript_token = config
        .ai
        .transcription
        .ai_transcription_token
        .clone()
        .unwrap_or_default();
    let chat_token = config
        .ai
        .question
        .ai_question_token
        .clone()
        .unwrap_or_default();
    error_message = error_message
        .replace(&discord_token, "[REDACTED]")
        .replace(&image_token, "[REDACTED]")
        .replace(&transcript_token, "[REDACTED]")
        .replace(&chat_token, "[REDACTED]")
        .replace(&db_user, "[REDACTED]")
        .replace(&db_pass, "[REDACTED]")
        .replace(&db_port, "[REDACTED]")
        .replace(&db_host, "[REDACTED]");
    // replace url with [REDACTED]
    let url_regex = Regex::new(r"https?://(www\.)?[-a-zA-Z0-9@:%._+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_+.~#?&/=]*)").unwrap();
    error_message = url_regex
        .replace_all(&error_message, "[REDACTED]")
        .to_string();

    error_message
}

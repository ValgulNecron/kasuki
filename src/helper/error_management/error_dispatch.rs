use regex::Regex;
use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseFollowup, CreateInteractionResponseMessage, Timestamp,
};
use tracing::error;

use crate::constant::COLOR;
use crate::event_handler::Handler;
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

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
    error: AppError,
    command_interaction: &CommandInteraction,
    ctx: &Context,
    self_handler: &Handler,
) {
    error!("{}", error);
    let response_type = error.error_response_type.clone();
    match response_type {
        ErrorResponseType::Message => {
            match send_error(error, command_interaction, ctx, self_handler).await {
                Ok(_) => {}
                Err(e) => {
                    error!("{}", e);
                }
            }
        }
        ErrorResponseType::Followup => {
            match send_differed_error(error, command_interaction, ctx, self_handler).await {
                Ok(_) => {}
                Err(e) => {
                    error!("{}", e);
                }
            }
        }
        ErrorResponseType::Unknown => {
            match send_error(error.clone(), command_interaction, ctx, self_handler).await {
                Ok(_) => {}
                Err(_) => {
                    match send_differed_error(error, command_interaction, ctx, self_handler).await {
                        Ok(_) => {}
                        Err(e) => {
                            error!("{}", e);
                        }
                    }
                }
            }
        }
        ErrorResponseType::None => {}
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
    e: AppError,
    command_interaction: &CommandInteraction,
    ctx: &Context,
    self_handler: &Handler,
) -> Result<(), AppError> {
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
        .map_err(|e| {
            AppError::new(
                format!("Error while sending the command {}", e),
                ErrorType::Command,
                ErrorResponseType::Message,
            )
        })?;
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
    e: AppError,
    command_interaction: &CommandInteraction,
    ctx: &Context,
    self_handler: &Handler,
) -> Result<(), AppError> {
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
        .map_err(|e| {
            AppError::new(
                format!("Error while sending the command {}", e),
                ErrorType::Command,
                ErrorResponseType::Followup,
            )
        });

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
        .replace(&chat_token, "[REDACTED]");
    let re = Regex::new(r"^(https?://)[^/]+").unwrap();
    error_message = re.replace(&error_message, "$1[REDACTED]").to_string();

    error_message
}

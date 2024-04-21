use regex::Regex;
use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseFollowup, CreateInteractionResponseMessage, Timestamp,
};
use tracing::error;

use crate::constant::{CHAT_TOKEN, COLOR, DISCORD_TOKEN, IMAGE_TOKEN, TRANSCRIPT_TOKEN};
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

const ERROR_MESSAGE: &str = "**There was an error while processing the command**\
    \n**This error is most likely an input error** \
    **like searching for non existent anime, requesting nsfw image to the ai, etc.**\n \
    **but in some case it's a server or programming error**.\
    in this case report it to me and I will try to fix it the fastest I can**";

pub async fn command_dispatching(
    error: AppError,
    command_interaction: &CommandInteraction,
    ctx: &Context,
) {
    error!("{}", error);
    let response_type = error.error_response_type.clone();
    match response_type {
        ErrorResponseType::Message => match send_error(error, command_interaction, ctx).await {
            Ok(_) => {}
            Err(e) => {
                error!("{}", e);
            }
        },
        ErrorResponseType::Followup => {
            match send_differed_error(error, command_interaction, ctx).await {
                Ok(_) => {}
                Err(e) => {
                    error!("{}", e);
                }
            }
        }
        ErrorResponseType::Unknown => {
            match send_error(error.clone(), command_interaction, ctx).await {
                Ok(_) => {}
                Err(_) => match send_differed_error(error, command_interaction, ctx).await {
                    Ok(_) => {}
                    Err(e) => {
                        error!("{}", e);
                    }
                },
            }
        }
        ErrorResponseType::None => {}
    }
}

async fn send_error(
    e: AppError,
    command_interaction: &CommandInteraction,
    ctx: &Context,
) -> Result<(), AppError> {
    let error_message = format!("{}\n{}", ERROR_MESSAGE, e);
    // censor url and token in the error message
    let error_message = censor_url_and_token(error_message);
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

async fn send_differed_error(
    e: AppError,
    command_interaction: &CommandInteraction,
    ctx: &Context,
) -> Result<(), AppError> {
    let error_message = format!("{}\n{}", ERROR_MESSAGE, e);
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

fn censor_url_and_token(error_message: String) -> String {
    let mut error_message = error_message;
    let discord_token = DISCORD_TOKEN.to_string();
    let image_token = IMAGE_TOKEN.to_string();
    let transcript_token = TRANSCRIPT_TOKEN.to_string();
    let chat_token = CHAT_TOKEN.to_string();
    error_message = error_message
        .replace(&discord_token, "[REDACTED]")
        .replace(&image_token, "[REDACTED]")
        .replace(&transcript_token, "[REDACTED]")
        .replace(&chat_token, "[REDACTED]");
    let re = Regex::new(r"^(https?://)[^/]+").unwrap();
    error_message = re.replace(&error_message, "$1[REDACTED]").to_string();

    error_message
}

use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseFollowup, CreateInteractionResponseMessage, Timestamp,
};
use tracing::error;

use crate::constant::COLOR;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

pub async fn command_dispatching(
    error: AppError,
    command_interaction: &CommandInteraction,
    ctx: &Context,
) {
    error!("{:?}", error);
    let response_type = error.error_response_type.clone();
    match response_type {
        ErrorResponseType::Message => match send_error(error, command_interaction, ctx).await {
            Ok(_) => {}
            Err(e) => {
                error!("{:?}", e);
            }
        },
        ErrorResponseType::Followup => {
            match send_differed_error(error, command_interaction, ctx).await {
                Ok(_) => {}
                Err(e) => {
                    error!("{:?}", e);
                }
            }
        }
        ErrorResponseType::Unknown => {
            match send_error(error.clone(), command_interaction, ctx).await {
                Ok(_) => {}
                Err(_) => match send_differed_error(error, command_interaction, ctx).await {
                    Ok(_) => {}
                    Err(e) => {
                        error!("{:?}", e);
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
    let error_message = format!("**This error is most likely an error on your part. \
    like you asking the bot to find unknown stuff or other. but in some case it's an error on my part juts check the \
    error and report it to me and I will try to fix it the fastest I can**  \n{:?}", e);
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
    let error_message = format!("**This error is most likely an error on your part. \
    like you asking the bot to find unknown stuff or other. but in some case it's an error on my part juts check the \
    error and report it to me and I will try to fix it the fastest I can**  \n{:?}", e);
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

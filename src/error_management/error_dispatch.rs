use crate::constant::COLOR;
use serenity::all::{
    CommandInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseFollowup, CreateInteractionResponseMessage, Timestamp,
};
use tracing::error;

use crate::error_enum::{AppError, DifferedError, Error};

pub async fn command_dispatching(
    error: AppError,
    command_interaction: &CommandInteraction,
    ctx: &Context,
) {
    error!("{:?}", error);
    match error {
        AppError::Error(e) => send_error(e, command_interaction, ctx).await,
        AppError::DifferedError(e) => send_differed_error(e, command_interaction, ctx).await,
        AppError::ComponentError(_) => {}
        AppError::NotACommandError(_) => {}
        AppError::JoiningError(_) => {}
    }
}

async fn send_error(e: Error, command_interaction: &CommandInteraction, ctx: &Context) {
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

    let _ = command_interaction
        .create_response(&ctx.http, builder)
        .await;
}

async fn send_differed_error(
    e: DifferedError,
    command_interaction: &CommandInteraction,
    ctx: &Context,
) {
    let error_message = format!("Please verify the error bellow and contact me on discord or github depending on the error: \n{:?}", e);
    let builder_embed = CreateEmbed::new()
        .timestamp(Timestamp::now())
        .color(COLOR)
        .description(error_message)
        .title("There was an error while processing the command");

    let builder = CreateInteractionResponseFollowup::new().embed(builder_embed);

    let _ = command_interaction
        .create_followup(&ctx.http, builder)
        .await;
}

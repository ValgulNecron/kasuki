use crate::constant::{COLOR, COMMAND_SENDING_ERROR, OPTION_ERROR};
use crate::error_enum::AppError;
use crate::error_enum::AppError::LangageGuildIdError;
use crate::structure::general::avatar::load_localization_avatar;
use serenity::all::CreateInteractionResponse::Defer;
use serenity::all::{
    CommandDataOption, CommandInteraction, Context, CreateInteractionResponseMessage,
};

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &CommandInteraction,
) -> Result<(), AppError> {
    let guild_id = command
        .guild_id
        .ok_or(LangageGuildIdError(String::from(
            "Guild id for langage not found.",
        )))?
        .to_string();

    let image_localised = load_localization_avatar(guild_id).await?;

    let builder_message = Defer()

    command
        .create_response(&ctx.http, builder_message)
        .await
        .map_err(|_| COMMAND_SENDING_ERROR.clone())
}

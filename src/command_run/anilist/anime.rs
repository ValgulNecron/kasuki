use crate::anilist_struct::run::media::{send_embed, MediaWrapper};
use crate::constant::OPTION_ERROR;
use crate::error_enum::AppError;
use crate::error_enum::AppError::NoCommandOption;
use serenity::all::{CommandDataOption, CommandDataOptionValue, CommandInteraction, Context};
use tracing::trace;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &CommandInteraction,
) -> Result<(), AppError> {
    let mut value = String::new();
    for option_data in options {
        if option_data.name.as_str() != "type" {
            let option_value = option_data.value.as_str().clone().unwrap();
            value = option_value.to_string().clone()
        }
    }

    let data: MediaWrapper = if value.parse::<i32>().is_ok() {
        MediaWrapper::new_anime_by_id(value.parse().unwrap()).await?
    } else {
        MediaWrapper::new_anime_by_search(&value).await?
    };

    send_embed(ctx, command, data).await
}

use crate::anilist_struct::run::media::{send_embed, MediaWrapper};
use crate::constant::OPTION_ERROR;
use crate::error_enum::AppError;
use crate::error_enum::AppError::NoCommandOption;
use serenity::all::{CommandDataOption, CommandDataOptionValue, CommandInteraction, Context};

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &CommandInteraction,
) -> Result<(), AppError> {
    let option = &options.get(0).ok_or(OPTION_ERROR.clone())?.value;

    let value = match option {
        CommandDataOptionValue::String(lang) => lang,
        _ => {
            return Err(NoCommandOption(String::from(
                "The command contain no option.",
            )));
        }
    };

    let data: MediaWrapper = if value.parse::<i32>().is_ok() {
        MediaWrapper::new_anime_by_id(value.parse().unwrap()).await?
    } else {
        MediaWrapper::new_anime_by_search(value).await?
    };

    send_embed(ctx, command, data).await
}

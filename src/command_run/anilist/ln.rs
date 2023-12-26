use serenity::all::{CommandDataOption, CommandInteraction, Context};

use crate::anilist_struct::run::media::{send_embed, MediaWrapper};
use crate::error_enum::AppError;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command: &CommandInteraction,
) -> Result<(), AppError> {
    let mut value = String::new();
    for option_data in options {
        if option_data.name.as_str() != "type" {
            let option_value = option_data.value.as_str().unwrap();
            value = option_value.to_string().clone()
        }
    }

    let data: MediaWrapper = if value.parse::<i32>().is_ok() {
        MediaWrapper::new_ln_by_id(value.parse().unwrap()).await?
    } else {
        MediaWrapper::new_ln_by_search(&value).await?
    };

    send_embed(ctx, command, data).await
}

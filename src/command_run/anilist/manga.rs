use serenity::all::{CommandDataOption, CommandInteraction, Context};

use crate::anilist_struct::run::media::{send_embed, MediaWrapper};
use crate::common::get_option_value::get_option;
use crate::error_management::error_enum::AppError;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command_interaction: &CommandInteraction,
) -> Result<(), AppError> {
    let value = get_option(options);

    let data: MediaWrapper = if value.parse::<i32>().is_ok() {
        MediaWrapper::new_manga_by_id(value.parse().unwrap()).await?
    } else {
        MediaWrapper::new_manga_by_search(&value).await?
    };

    send_embed(ctx, command_interaction, data).await
}

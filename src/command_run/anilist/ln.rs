use serenity::all::{CommandInteraction, Context};

use crate::anilist_struct::run::media::{send_embed, MediaWrapper};
use crate::command_run::get_option::get_option_map_string;
use crate::error_management::error_enum::AppError;

pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    let map = get_option_map_string(command_interaction);
    let value = map
        .get(&String::from("ln_name"))
        .cloned()
        .unwrap_or(String::new());

    let data: MediaWrapper = if value.parse::<i32>().is_ok() {
        MediaWrapper::new_ln_by_id(value.parse().unwrap()).await?
    } else {
        MediaWrapper::new_ln_by_search(&value).await?
    };

    send_embed(ctx, command_interaction, data).await
}

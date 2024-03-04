use serenity::all::{CommandInteraction, Context};

use crate::anilist_struct::run::media::{send_embed, MediaWrapper};
use crate::command_run::get_option::get_option_map_string_subcommand;
use crate::error_management::error_enum::AppError;

pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    let map = get_option_map_string_subcommand(command_interaction);
    let value = map
        .get(&String::from("anime_name"))
        .cloned()
        .unwrap_or(String::new());

    let data: MediaWrapper = if value.parse::<i32>().is_ok() {
        MediaWrapper::new_anime_by_id(value.parse().unwrap()).await?
    } else {
        MediaWrapper::new_anime_by_search(&value).await?
    };

    send_embed(ctx, command_interaction, data).await
}

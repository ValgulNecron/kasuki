use serenity::all::{CommandInteraction, Context};

use crate::anilist_struct::run::media::{MediaWrapper, send_embed};
use crate::common::get_option::subcommand::get_option_map_string_subcommand;
use crate::error_management::error_enum::AppError;

pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    let map = get_option_map_string_subcommand(command_interaction);
    let value = map
        .get(&String::from("manga_name"))
        .cloned()
        .unwrap_or(String::new());

    let data: MediaWrapper = if value.parse::<i32>().is_ok() {
        MediaWrapper::new_manga_by_id(value.parse().unwrap()).await?
    } else {
        MediaWrapper::new_manga_by_search(&value).await?
    };

    send_embed(ctx, command_interaction, data).await
}

use serenity::all::{CommandInteraction, Context};

use crate::anilist_struct::run::character::{send_embed, CharacterWrapper};
use crate::command_run::get_option::get_option_map_string_subcommand;
use crate::error_management::error_enum::AppError;

pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    let map = get_option_map_string_subcommand(command_interaction);
    let value = map
        .get(&String::from("name"))
        .cloned()
        .unwrap_or(String::new());

    let data: CharacterWrapper = if value.parse::<i32>().is_ok() {
        CharacterWrapper::new_character_by_id(value.parse().unwrap()).await?
    } else {
        CharacterWrapper::new_character_by_search(&value).await?
    };

    send_embed(ctx, command_interaction, data).await
}

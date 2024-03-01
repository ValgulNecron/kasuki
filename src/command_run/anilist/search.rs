use serenity::all::{CommandInteraction, Context};

use crate::command_run::anilist::{anime, character, ln, manga, staff, studio, user};
use crate::command_run::get_option::get_option_map_string_subcommand;
use crate::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};

pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    let map = get_option_map_string_subcommand(command_interaction);
    let search_type = map.get(&String::from("type")).ok_or(AppError::new(
        String::from("There is no option"),
        ErrorType::Option,
        ErrorResponseType::Followup,
    ))?;
    match search_type.as_str() {
        "anime" => anime::run(ctx, command_interaction).await,
        "character" => character::run(ctx, command_interaction).await,
        "ln" => ln::run(ctx, command_interaction).await,
        "manga" => manga::run(ctx, command_interaction).await,
        "staff" => staff::run(ctx, command_interaction).await,
        "user" => user::run(ctx, command_interaction).await,
        "studio" => studio::run(ctx, command_interaction).await,
        _ => Err(AppError::new(
            String::from("Invalid type"),
            ErrorType::Option,
            ErrorResponseType::Followup,
        )),
    }
}

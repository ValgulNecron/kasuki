use serenity::all::{CommandDataOption, CommandInteraction, Context};

use crate::command_run::anilist::{anime, character, ln, manga, staff, studio, user};
use crate::error_management::error_enum::AppError;
use crate::error_management::error_enum::AppError::Error;
use crate::error_management::error_enum::CommandError::NotAValidTypeError;

pub async fn run(
    options: &[CommandDataOption],
    ctx: &Context,
    command_interaction: &CommandInteraction,
) -> Result<(), AppError> {
    let mut search_type = String::new();

    for option in options {
        if option.name.as_str() == "type" {
            search_type = option.value.as_str().unwrap().to_string()
        }
    }
    match search_type.as_str() {
        "anime" => anime::run(options, ctx, command_interaction).await,
        "character" => character::run(options, ctx, command_interaction).await,
        "ln" => ln::run(options, ctx, command_interaction).await,
        "manga" => manga::run(options, ctx, command_interaction).await,
        "staff" => staff::run(options, ctx, command_interaction).await,
        "user" => user::run(options, ctx, command_interaction).await,
        "studio" => studio::run(options, ctx, command_interaction).await,
        _ => Err(Error(NotAValidTypeError(String::from("Invalid type")))),
    }
}

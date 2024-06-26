use serenity::all::{CommandInteraction, Context};
use std::sync::Arc;

use crate::command::run::anilist_user::{anime, character, ln, manga, staff, studio, user};
use crate::config::Config;
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;

/// Executes the command to search for a specific type of AniList data.
///
/// This function retrieves the type of AniList data to search for from the command interaction and executes the corresponding search function.
/// The type of AniList data can be one of the following: "anime", "character", "ln", "manga", "staff", "user", or "studio".
/// If the specified type is not one of these, the function returns an error.
///
/// # Arguments
///
/// * `ctx` - The context in which this command is being executed.
/// * `command_interaction` - The interaction that triggered this command.
///
/// # Returns
///
/// A `Result` that is `Ok` if the command executed successfully, or `Err` if an error occurred.
pub async fn run(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
) -> Result<(), AppError> {
    // Retrieve the type of AniList data to search for from the command interaction
    let map = get_option_map_string_subcommand(command_interaction);
    let search_type = map.get(&String::from("type")).ok_or(AppError::new(
        String::from("There is no option"),
        ErrorType::Option,
        ErrorResponseType::Followup,
    ))?;

    // Execute the corresponding search function based on the specified type
    match search_type.as_str() {
        "anime" => anime::run(ctx, command_interaction).await,
        "character" => character::run(ctx, command_interaction).await,
        "ln" => ln::run(ctx, command_interaction).await,
        "manga" => manga::run(ctx, command_interaction).await,
        "staff" => staff::run(ctx, command_interaction).await,
        "user" => user::run(ctx, command_interaction).await,
        "studio" => studio::run(ctx, command_interaction).await,
        // Return an error if the specified type is not one of the expected types
        _ => Err(AppError::new(
            String::from("Invalid type"),
            ErrorType::Option,
            ErrorResponseType::Followup,
        )),
    }
}

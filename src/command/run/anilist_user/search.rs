use std::error::Error;
use std::sync::Arc;

use moka::future::Cache;
use serenity::all::{CommandInteraction, Context};
use tokio::sync::RwLock;

use crate::command::run::anilist_user::{anime, character, ln, manga, staff, studio, user};
use crate::config::Config;
use crate::helper::error_management::error_enum::ResponseError;
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
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<(), Box<dyn Error>> {
    // Retrieve the type of AniList data to search for from the command interaction
    let map = get_option_map_string_subcommand(command_interaction);
    let search_type = map
        .get(&String::from("type"))
        .ok_or(ResponseError::Option(String::from("No type specified")))?;

    // Execute the corresponding search function based on the specified type
    match search_type.as_str() {
        "anime" => anime::run(ctx, command_interaction, config, anilist_cache).await,
        "character" => character::run(ctx, command_interaction, config, anilist_cache).await,
        "ln" => ln::run(ctx, command_interaction, config, anilist_cache).await,
        "manga" => manga::run(ctx, command_interaction, config, anilist_cache).await,
        "staff" => staff::run(ctx, command_interaction, config, anilist_cache).await,
        "user" => user::run(ctx, command_interaction, config, anilist_cache).await,
        "studio" => studio::run(ctx, command_interaction, config, anilist_cache).await,
        // Return an error if the specified type is not one of the expected types
        _ => Err(Box::new(ResponseError::Option(String::from(
            "Type does not exist.",
        )))),
    }
}

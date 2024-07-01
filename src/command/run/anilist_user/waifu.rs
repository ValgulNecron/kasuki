use std::sync::Arc;

use moka::future::Cache;
use serenity::all::{CommandInteraction, Context};
use tokio::sync::RwLock;

use crate::command::run::anilist_user::character::get_character_by_id;
use crate::config::Config;
use crate::helper::error_management::error_enum::AppError;
use crate::structure::run::anilist::character::send_embed;

/// Executes the command to fetch and display information about a specific character from AniList.
///
/// This function fetches the data of a character with a specific ID from AniList and sends it as a response to the command interaction.
/// The character ID is currently hardcoded as 156323.
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
) -> Result<(), AppError> {
    let db_type = config.bot.config.db_type.clone();
    // Fetch the data of the character with ID 156323 from AniList
    let value = 156323;
    let data = get_character_by_id(value, anilist_cache).await?;
    // Send the character's data as a response to the command interaction
    send_embed(ctx, command_interaction, data, db_type).await
}

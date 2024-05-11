use serenity::all::{CommandInteraction, Context};

use crate::anilist_struct::run::character::{send_embed, CharacterWrapper};
use crate::helper::error_management::error_enum::AppError;

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
pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    // Fetch the data of the character with ID 156323 from AniList
    let data: CharacterWrapper = CharacterWrapper::new_character_by_id(156323).await?;

    // Send the character's data as a response to the command interaction
    send_embed(ctx, command_interaction, data).await
}

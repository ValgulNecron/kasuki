use serenity::all::{CommandInteraction, Context};

use crate::anilist_struct::run::character::{send_embed, CharacterWrapper};
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::helper::error_management::error_enum::AppError;

/// This asynchronous function runs the command interaction for retrieving information about a character.
///
/// It first retrieves the name or ID of the character from the command interaction options.
///
/// If the value is an integer, it treats it as an ID and retrieves the character with that ID.
/// If the value is not an integer, it treats it as a name and retrieves the character with that name.
///
/// It sends an embed with the character information as a response to the command interaction.
///
/// # Arguments
///
/// * `ctx` - The context in which this function is being called.
/// * `command_interaction` - The command interaction that triggered this function.
///
/// # Returns
///
/// A `Result` indicating whether the function executed successfully. If an error occurred, it contains an `AppError`.
pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    // Retrieve the name or ID of the character from the command interaction options
    let map = get_option_map_string_subcommand(command_interaction);
    let value = map
        .get(&String::from("name"))
        .cloned()
        .unwrap_or(String::new());

    // If the value is an integer, treat it as an ID and retrieve the character with that ID
    // If the value is not an integer, treat it as a name and retrieve the character with that name
    let data: CharacterWrapper = if value.parse::<i32>().is_ok() {
        CharacterWrapper::new_character_by_id(value.parse().unwrap()).await?
    } else {
        CharacterWrapper::new_character_by_search(&value).await?
    };

    // Send an embed with the character information as a response to the command interaction
    send_embed(ctx, command_interaction, data).await
}

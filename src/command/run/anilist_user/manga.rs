use serenity::all::{CommandInteraction, Context};

use crate::anilist_struct::run::media::{send_embed, MediaWrapper};
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::error_management::error_enum::AppError;

/// Executes the command to fetch and display information about a manga based on its name or ID.
///
/// This function retrieves the name or ID of the manga from the command interaction. If the value can be parsed as an `i32`, it is treated as an ID and the function fetches the manga data by ID.
/// If the value cannot be parsed as an `i32`, it is treated as a name and the function fetches the manga data by search.
/// The function then sends an embed containing the manga data as a response to the command interaction.
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
    // Retrieve the name or ID of the manga from the command interaction
    let map = get_option_map_string_subcommand(command_interaction);
    let value = map
        .get(&String::from("manga_name"))
        .cloned()
        .unwrap_or(String::new());

    // Fetch the manga data by ID if the value can be parsed as an `i32`, or by search otherwise
    let data: MediaWrapper = if value.parse::<i32>().is_ok() {
        MediaWrapper::new_manga_by_id(value.parse().unwrap()).await?
    } else {
        MediaWrapper::new_manga_by_search(&value).await?
    };

    // Send an embed containing the manga data as a response to the command interaction
    send_embed(ctx, command_interaction, data).await
}

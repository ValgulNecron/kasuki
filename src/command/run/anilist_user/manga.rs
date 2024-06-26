use crate::config::Config;
use crate::helper::error_management::error_enum::AppError;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::run::anilist::media::{
    send_embed, Media, MediaFormat, MediaQuerryId, MediaQuerryIdVariables, MediaQuerrySearch,
    MediaQuerrySearchVariables, MediaType,
};
use cynic::{GraphQlResponse, QueryBuilder};
use serenity::all::{CommandInteraction, Context};
use std::sync::Arc;

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
pub async fn run(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
) -> Result<(), AppError> {
    // Retrieve the name or ID of the manga from the command interaction
    let map = get_option_map_string_subcommand(command_interaction);
    let value = map
        .get(&String::from("manga_name"))
        .cloned()
        .unwrap_or(String::new());

    // Fetch the manga data by ID if the value can be parsed as an `i32`, or by search otherwise
    let data: Media = if value.parse::<i32>().is_ok() {
        let id = value.parse::<i32>().unwrap();
        let var = MediaQuerryIdVariables {
            format_in: Some(vec![Some(MediaFormat::OneShot), Some(MediaFormat::Manga)]),
            id: Some(id),
            media_type: Some(MediaType::Manga),
        };

        let operation = MediaQuerryId::build(var);
        let data: GraphQlResponse<MediaQuerryId> = make_request_anilist(operation, false).await?;
        data.data.unwrap().media.unwrap()
    } else {
        let var = MediaQuerrySearchVariables {
            format_in: Some(vec![Some(MediaFormat::OneShot), Some(MediaFormat::Manga)]),
            search: Some(&*value),
            media_type: Some(MediaType::Manga),
        };
        let operation = MediaQuerrySearch::build(var);
        let data: GraphQlResponse<MediaQuerrySearch> =
            make_request_anilist(operation, false).await?;
        data.data.unwrap().media.unwrap()
    };

    // Send an embed containing the manga data as a response to the command interaction
    send_embed(ctx, command_interaction, data).await
}

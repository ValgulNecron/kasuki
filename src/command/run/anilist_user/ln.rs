use std::error::Error;
use std::sync::Arc;

use cynic::{GraphQlResponse, QueryBuilder};
use moka::future::Cache;
use serenity::all::{CommandInteraction, Context};
use tokio::sync::RwLock;

use crate::config::Config;
use crate::helper::get_option::command::get_option_map_string;
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::run::anilist::media::{
    send_embed, Media, MediaFormat, MediaQuerryId, MediaQuerryIdVariables, MediaQuerrySearch,
    MediaQuerrySearchVariables, MediaType,
};

/// Executes the command to fetch and display information about a light novel (LN) based on its name or ID.
///
/// This function retrieves the name or ID of the LN from the command interaction. If the value can be parsed as an `i32`, it is treated as an ID and the function fetches the LN data by ID.
/// If the value cannot be parsed as an `i32`, it is treated as a name and the function fetches the LN data by search.
/// The function then sends an embed containing the LN data as a response to the command interaction.
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
    let db_type = config.bot.config.db_type.clone();
    // Retrieve the name or ID of the LN from the command interaction
    let map = get_option_map_string(command_interaction);
    let value = map
        .get(&String::from("ln_name"))
        .cloned()
        .unwrap_or(String::new());

    // Fetch the LN data by ID if the value can be parsed as an `i32`, or by search otherwise
    let data: Media = if value.parse::<i32>().is_ok() {
        let id = value.parse::<i32>().unwrap();
        let var = MediaQuerryIdVariables {
            format_in: Some(vec![Some(MediaFormat::Novel)]),
            id: Some(id),
            media_type: Some(MediaType::Manga),
        };
        let operation = MediaQuerryId::build(var);
        let data: GraphQlResponse<MediaQuerryId> =
            make_request_anilist(operation, false, anilist_cache).await?;
        data.data.unwrap().media.unwrap()
    } else {
        let var = MediaQuerrySearchVariables {
            format_in: Some(vec![Some(MediaFormat::Novel)]),
            search: Some(&*value),
            media_type: Some(MediaType::Manga),
        };

        let operation = MediaQuerrySearch::build(var);
        let data: GraphQlResponse<MediaQuerrySearch> =
            make_request_anilist(operation, false, anilist_cache).await?;
        data.data.unwrap().media.unwrap()
    };

    // Send an embed containing the LN data as a response to the command interaction
    send_embed(
        ctx,
        command_interaction,
        data,
        db_type,
        config.bot.config.clone(),
    )
    .await?;

    Ok(())
}

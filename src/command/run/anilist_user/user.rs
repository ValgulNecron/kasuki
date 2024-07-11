use std::sync::Arc;

use cynic::{GraphQlResponse, QueryBuilder};
use moka::future::Cache;
use serenity::all::{CommandInteraction, Context};
use tokio::sync::RwLock;

use crate::config::Config;
use crate::database::data_struct::registered_user::RegisteredUser;
use crate::database::manage::dispatcher::data_dispatch::get_registered_user;
use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::run::anilist::user::{
    send_embed, User, UserQuerryId, UserQuerryIdVariables, UserQuerrySearch,
    UserQuerrySearchVariables,
};

/// Executes the command to fetch and display information about a user from AniList.
///
/// This function retrieves the username from the command interaction and fetches the user's data from AniList.
/// If the username is not provided, it fetches the data of the user who triggered the command interaction.
/// It then sends the user's data as a response to the command interaction.
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
    // Retrieve the username from the command interaction
    let map = get_option_map_string_subcommand(command_interaction);
    let user = map.get(&String::from("username"));

    // If the username is provided, fetch the user's data from AniList and send it as a response
    if let Some(value) = user {
        let data: User = get_user(value, anilist_cache.clone()).await?;
        return send_embed(ctx, command_interaction, data, db_type.clone()).await;
    }

    // If the username is not provided, fetch the data of the user who triggered the command interaction
    let user_id = &command_interaction.user.id.to_string();
    let row: Option<RegisteredUser> = get_registered_user(user_id, db_type.clone()).await?;
    let user = row.ok_or(AppError::new(
        String::from("There is no option"),
        ErrorType::Option,
        ErrorResponseType::Followup,
    ))?;

    // Fetch the user's data from AniList and send it as a response
    let data = get_user(&user.anilist_id, anilist_cache).await?;
    send_embed(ctx, command_interaction, data, db_type).await
}
/// Fetches the data of a user from AniList.
///
/// This function takes a username or user ID and fetches the user's data from AniList.
/// If the username or user ID is not valid, it returns an error.
///
/// # Arguments
///
/// * `value` - The username or user ID of the user.
///
/// # Returns
///
/// A `Result` that is `Ok` if the user's data was fetched successfully, or `Err` if an error occurred.
pub async fn get_user(
    value: &str,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<User, AppError> {
    // If the value is a valid user ID, fetch the user's data by ID
    let user = if value.parse::<i32>().is_ok() {
        let id = value.parse::<i32>().unwrap();
        let var = UserQuerryIdVariables { id: Some(id) };
        let operation = UserQuerryId::build(var);
        let data: GraphQlResponse<UserQuerryId> =
            make_request_anilist(operation, false, anilist_cache).await?;
        data.data.unwrap().user.unwrap()
    } else {
        // If the value is not a valid user ID, fetch the user's data by username
        let var = UserQuerrySearchVariables {
            search: Some(value),
        };
        let operation = UserQuerrySearch::build(var);
        let data: GraphQlResponse<UserQuerrySearch> =
            make_request_anilist(operation, false, anilist_cache).await?;
        data.data.unwrap().user.unwrap()
    };
    Ok(user)
}

use std::error::Error;
use std::sync::Arc;

use crate::command::command_trait::{Command, SlashCommand};
use crate::config::Config;
use crate::get_url;
use crate::helper::error_management::error_dispatch;
use crate::helper::get_option::command::get_option_map_string;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::database::prelude::RegisteredUser;
use crate::structure::database::registered_user::Column;
use crate::structure::run::anilist::user;
use crate::structure::run::anilist::user::{
    User, UserQuerryId, UserQuerryIdVariables, UserQuerrySearch, UserQuerrySearchVariables,
};
use cynic::{GraphQlResponse, QueryBuilder};
use moka::future::Cache;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use serenity::all::{CommandInteraction, Context};
use tokio::sync::RwLock;

pub struct UserCommand {
    pub ctx: Context,
    pub command_interaction: CommandInteraction,
    pub config: Arc<Config>,
    pub anilist_cache: Arc<RwLock<Cache<String, String>>>,
}

impl Command for UserCommand {
    fn get_ctx(&self) -> &Context {
        &self.ctx
    }
    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for UserCommand {
    async fn run_slash(&self) -> Result<(), Box<dyn Error>> {
        let ctx = &self.ctx;
        let command_interaction = &self.command_interaction;
        let config = self.config.clone();
        let anilist_cache = self.anilist_cache.clone();
        send_embed(ctx, command_interaction, config, anilist_cache).await
    }
}
async fn send_embed(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    config: Arc<Config>,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
) -> Result<(), Box<dyn Error>> {
    let db_type = config.bot.config.db_type.clone();
    // Retrieve the username from the command interaction
    let map = get_option_map_string(command_interaction);
    let user = map.get(&String::from("username"));

    // If the username is provided, fetch the user's data from AniList and send it as a response
    if let Some(value) = user {
        let data: User = get_user(value, anilist_cache.clone()).await?;
        return user::send_embed(ctx, command_interaction, data, config.bot.config.clone()).await;
    }

    // If the username is not provided, fetch the data of the user who triggered the command interaction
    let user_id = &command_interaction.user.id.to_string();
    let connection = sea_orm::Database::connect(get_url(config.bot.config.clone())).await?;
    let row = RegisteredUser::find()
        .filter(Column::UserId.eq(user_id))
        .one(&connection)
        .await?;
    let user = row.ok_or(error_dispatch::Error::Option(String::from("No user found")))?;

    // Fetch the user's data from AniList and send it as a response
    let data = get_user(user.anilist_id.to_string().as_str(), anilist_cache).await?;
    user::send_embed(ctx, command_interaction, data, config.bot.config.clone()).await
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
) -> Result<User, Box<dyn Error>> {
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

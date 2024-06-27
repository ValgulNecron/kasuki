use cynic::{GraphQlResponse, QueryBuilder};
use serenity::all::{
    AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
    CreateInteractionResponse,
};

use crate::constant::DEFAULT_STRING;
use crate::helper::error_management::error_enum::AppError;
use crate::helper::get_option::subcommand::get_option_map_string_autocomplete_subcommand;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::autocomplete::anilist::user::{UserAutocomplete, UserAutocompleteVariables};
use moka::future::Cache;
use std::sync::Arc;
use tokio::sync::RwLock;

/// `autocomplete` is an asynchronous function that handles the autocomplete feature for user search.
/// It takes a `Context` and a `CommandInteraction` as parameters.
/// `ctx` is the context in which this function is called.
/// `autocomplete_interaction` is the command interaction that triggered this function.
///
/// This function first gets the map of options from the command interaction.
/// It then gets the username from the map of options.
/// If the username is not found in the map, it defaults to a predefined string.
/// It then creates a new `UserPageWrapper` for the autocomplete user search with the username.
/// It gets the users from the `UserPageWrapper`.
/// For each user, it gets the name and creates a new `AutocompleteChoice` with the name and the ID and pushes it to the vector.
/// It creates a new `CreateAutocompleteResponse` with the choices.
/// It creates a new `CreateInteractionResponse` with the `CreateAutocompleteResponse`.
/// It sends the response to the Discord channel.
///
/// # Arguments
///
/// * `ctx` - The context in which this function is called.
/// * `autocomplete_interaction` - The command interaction that triggered this function.
///
/// # Async
///
/// This function is asynchronous. It awaits the creation of the `UserPageWrapper` and the sending of the response.
pub async fn autocomplete(
    ctx: Context,
    autocomplete_interaction: CommandInteraction,
    anilist_cache: Arc<RwLock<Cache<String, String>>>,
) {
    let map = get_option_map_string_autocomplete_subcommand(&autocomplete_interaction);
    let user_search = map.get(&String::from("username")).unwrap_or(DEFAULT_STRING);
    let var = UserAutocompleteVariables {
        search: Some(user_search),
    };
    let operation = UserAutocomplete::build(var);
    let data: Result<GraphQlResponse<UserAutocomplete>, AppError> =
        make_request_anilist(operation, false, anilist_cache).await;
    let data = match data {
        Ok(data) => data,
        Err(e) => {
            tracing::error!(?e);
            return;
        }
    };
    let users = data.data.unwrap().page.unwrap().users.unwrap();
    let mut choices = Vec::new();

    for user in users {
        let data = user.unwrap();
        let user = data.name;
        choices.push(AutocompleteChoice::new(user, data.id.to_string()))
    }

    let data = CreateAutocompleteResponse::new().set_choices(choices);
    let builder = CreateInteractionResponse::Autocomplete(data);

    let _ = autocomplete_interaction
        .create_response(ctx.http, builder)
        .await;
}

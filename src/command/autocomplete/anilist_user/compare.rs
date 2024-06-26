use cynic::{GraphQlResponse, QueryBuilder};
use serenity::all::{
    AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
    CreateInteractionResponse,
};
use tracing::log::trace;

use crate::constant::DEFAULT_STRING;
use crate::helper::error_management::error_enum::AppError;
use crate::helper::get_option::subcommand::get_option_map_string_autocomplete_subcommand;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::autocomplete::anilist::user::{UserAutocomplete, UserAutocompleteVariables};

/// `autocomplete` is an asynchronous function that handles the autocomplete feature for user comparison.
/// It takes a `Context` and a `CommandInteraction` as parameters.
/// `ctx` is the context in which this function is called.
/// `autocomplete_interaction` is the command interaction that triggered this function.
///
/// This function first gets the map of options from the command interaction.
/// It then gets the usernames from the map of options.
/// If the usernames are not found in the map, they default to a predefined string.
/// It then gets the autocomplete choices for each username.
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
/// This function is asynchronous. It awaits the getting of the choices and the sending of the response.
pub async fn autocomplete(
    ctx: Context,
    autocomplete_interaction: CommandInteraction,
    cache_type: String,
) {
    let mut choice = Vec::new();
    trace!("{:?}", &autocomplete_interaction.data.options);
    let map = get_option_map_string_autocomplete_subcommand(&autocomplete_interaction);
    let user1 = map.get(&String::from("username")).unwrap_or(DEFAULT_STRING);
    choice.extend(get_choices(user1, cache_type.clone()).await);
    let user2 = map
        .get(&String::from("username2"))
        .unwrap_or(DEFAULT_STRING);
    choice.extend(get_choices(user2, cache_type).await);

    let data = CreateAutocompleteResponse::new().set_choices(choice);
    let builder = CreateInteractionResponse::Autocomplete(data);

    let _ = autocomplete_interaction
        .create_response(ctx.http, builder)
        .await;
}

/// `get_choices` is an asynchronous function that gets the autocomplete choices for a username.
/// It takes a `search` as a parameter.
/// `search` is a string that represents the username.
/// It returns a vector of `AutocompleteChoice` that represents the autocomplete choices for the username.
///
/// This function first creates a new `UserPageWrapper` for the autocomplete user search with the username.
/// It then gets the users from the `UserPageWrapper`.
/// For each user, it gets the name and the ID.
/// It creates a new `AutocompleteChoice` with the name and the ID and pushes it to the vector.
///
/// # Arguments
///
/// * `search` - A string that represents the username.
///
/// # Returns
///
/// * `Vec<AutocompleteChoice>` - A vector of `AutocompleteChoice` that represents the autocomplete choices for the username.
///
/// # Async
///
/// This function is asynchronous. It awaits the creation of the `UserPageWrapper`.
async fn get_choices(search: &str, cache_type: String) -> Vec<AutocompleteChoice> {
    trace!("{:?}", search);
    let var = UserAutocompleteVariables {
        search: Some(search),
    };
    let operation = UserAutocomplete::build(var);
    let data: Result<GraphQlResponse<UserAutocomplete>, AppError> =
        make_request_anilist(operation, false, cache_type).await;
    let data = match data {
        Ok(data) => data,
        Err(e) => {
            tracing::error!(?e);
            return Vec::new();
        }
    };
    let users = data.data.unwrap().page.unwrap().users.unwrap();
    let mut choices = Vec::new();

    for user in users {
        let data = user.unwrap();
        let user = data.name;
        choices.push(AutocompleteChoice::new(user, data.id.to_string()))
    }
    choices
}

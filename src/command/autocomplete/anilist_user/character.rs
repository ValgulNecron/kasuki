use cynic::{GraphQlResponse, QueryBuilder};
use serenity::all::{
    AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
    CreateInteractionResponse,
};
use tracing::trace;

use crate::constant::DEFAULT_STRING;
use crate::helper::error_management::error_enum::AppError;
use crate::helper::get_option::subcommand::get_option_map_string_autocomplete_subcommand;
use crate::helper::make_graphql_cached::make_request_anilist;
use crate::structure::autocomplete::anilist::character::{
    CharacterAutocomplete, CharacterAutocompleteVariables,
};

/// `autocomplete` is an asynchronous function that handles the autocomplete feature for character search.
/// It takes a `Context` and a `CommandInteraction` as parameters.
/// `ctx` is the context in which this function is called.
/// `autocomplete_interaction` is the command interaction that triggered this function.
///
/// This function first gets the map of options from the command interaction.
/// It then gets the character name from the map of options.
/// If the character name is not found in the map, it defaults to a predefined string.
/// It then creates a new `CharacterPageWrapper` for the autocomplete character search with the character name.
/// It creates a new vector for the autocomplete choices.
/// It gets the characters from the `CharacterPageWrapper`.
/// For each character, it gets the name and the ID.
/// It creates a new `AutocompleteChoice` with the name and the ID and pushes it to the vector.
/// It creates a new `CreateAutocompleteResponse` with the vector of choices.
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
/// This function is asynchronous. It awaits the creation of the `CharacterPageWrapper`, the creation of the `CreateAutocompleteResponse`, and the sending of the response.
pub async fn autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
    let map = get_option_map_string_autocomplete_subcommand(&autocomplete_interaction);
    let character_search = map.get(&String::from("name")).unwrap_or(DEFAULT_STRING);
    let var = CharacterAutocompleteVariables {
        search: Some(character_search.as_str()),
    };
    let operation = CharacterAutocomplete::build(var);
    let data: Result<GraphQlResponse<CharacterAutocomplete>, AppError> =
        make_request_anilist(operation, false).await;
    let data = match data {
        Ok(data) => data,
        Err(e) => {
            tracing::error!(?e);
            return;
        }
    };
    trace!(?data);
    let mut choices = Vec::new();
    let characters = data.data.unwrap().page.unwrap().characters.unwrap();

    for character in characters {
        let data = character.unwrap();
        let name = data.name.unwrap();
        let full = name.full.clone();
        let user_pref = name.user_preferred.clone();
        let native = name.native.clone();
        let name = user_pref.unwrap_or(full.unwrap_or(native.unwrap_or(DEFAULT_STRING.clone())));
        choices.push(AutocompleteChoice::new(name, data.id.to_string()))
    }

    let data = CreateAutocompleteResponse::new().set_choices(choices);
    let builder = CreateInteractionResponse::Autocomplete(data);

    let _ = autocomplete_interaction
        .create_response(ctx.http, builder)
        .await;
}

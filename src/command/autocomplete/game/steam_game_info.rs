use rust_fuzzy_search::fuzzy_search_best_n;
use serenity::all::{
    AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
    CreateInteractionResponse,
};

use crate::helper::get_option::subcommand_group::get_option_map_string_subcommand_group;
use crate::constant::{APPS, AUTOCOMPLETE_COUNT_LIMIT, DEFAULT_STRING};

/// `autocomplete` is an asynchronous function that handles the autocomplete feature for game search.
/// It takes a `Context` and a `CommandInteraction` as parameters.
/// `ctx` is the context in which this function is called.
/// `autocomplete_interaction` is the command interaction that triggered this function.
///
/// This function first gets the map of options from the command interaction.
/// It then gets the game name from the map of options.
/// If the game name is not found in the map, it defaults to a predefined string.
/// It then creates a vector of app names from the global `APPS` array.
/// It performs a fuzzy search on the app names with the game name and a limit of `AUTOCOMPLETE_COUNT_LIMIT`.
/// It creates a new `AutocompleteChoice` for each matched string with the name and the ID and pushes it to the vector.
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
/// This function is asynchronous. It awaits the sending of the response.
///
/// # Safety
///
/// This function uses `unsafe` to access the global `APPS` array. The safety of this function depends on the correct usage of the `APPS` array.
pub async fn autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
    let map = get_option_map_string_subcommand_group(&autocomplete_interaction);
    let game_search = map
        .get(&String::from("game_name"))
        .unwrap_or(DEFAULT_STRING);

    let app_names: Vec<&str> = unsafe { APPS.iter().map(|app| app.0.as_str()).collect() };
    let result = fuzzy_search_best_n(
        game_search.as_str(),
        &app_names,
        AUTOCOMPLETE_COUNT_LIMIT as usize,
    );
    let mut choices: Vec<AutocompleteChoice> = Vec::new();

    // Map the indices of the matched strings back to their original positions in the APPS array
    for (data, _) in result {
        unsafe { choices.push(AutocompleteChoice::new(data, APPS[data].to_string())) }
    }

    let data = CreateAutocompleteResponse::new().set_choices(choices);
    let builder = CreateInteractionResponse::Autocomplete(data);

    let _ = autocomplete_interaction
        .create_response(ctx.http, builder)
        .await;
}

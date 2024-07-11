use serenity::all::{
    AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
    CreateInteractionResponse,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

use crate::constant::{AUTOCOMPLETE_COUNT_LIMIT, DEFAULT_STRING};
use crate::helper::fuzzy_search::distance_top_n;
use crate::helper::get_option::subcommand::get_option_map_string_autocomplete_subcommand;

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
pub async fn autocomplete(
    ctx: Context,
    autocomplete_interaction: CommandInteraction,
    apps: Arc<RwLock<HashMap<String, u128>>>,
) {
    let map = get_option_map_string_autocomplete_subcommand(&autocomplete_interaction);
    let game_search = map
        .get(&String::from("game_name"))
        .unwrap_or(DEFAULT_STRING);

    let guard = apps.read().await;
    let app_names: Vec<String> = guard.clone().iter().map(|app| app.0.clone()).collect();
    let app_names: Vec<&str> = app_names.iter().map(|a| a.as_str()).collect();
    let result = distance_top_n(
        game_search.as_str(),
        app_names.clone(),
        AUTOCOMPLETE_COUNT_LIMIT as usize,
    );
    debug!("Result: {:?}", result);
    let mut choices: Vec<AutocompleteChoice> = Vec::new();

    // truncate the name are longer than 0 characters and less than 100 characters
    // to prevent the discord api from returning an error
    if !result.is_empty() {
        for (name, _) in result {
            let name_show = if name.len() > 100 {
                // first 100 characters
                name.chars().take(100).collect::<String>()
            } else {
                name.clone()
            };
            if !name.is_empty() {
                choices.push(AutocompleteChoice::new(
                    name_show.clone(),
                    guard[&name].to_string(),
                ));
            }
        }
    }
    debug!("Choices: {:?}", choices.len());
    debug!("Choices: {:?}", choices);

    let data = CreateAutocompleteResponse::new().set_choices(choices);
    let builder = CreateInteractionResponse::Autocomplete(data);

    if let Err(why) = autocomplete_interaction
        .create_response(ctx.http, builder)
        .await
    {
        debug!("Error sending response: {:?}", why);
    }
}

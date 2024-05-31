use serenity::all::{
    AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
    CreateInteractionResponse,
};

use crate::constant::{AUTOCOMPLETE_COUNT_LIMIT, DEFAULT_STRING};
use crate::database::manage::dispatcher::data_dispatch::get_data_all_activity_by_server;
use crate::helper::get_option::subcommand_group::get_option_map_string_autocomplete_subcommand_group;

/// `autocomplete` is an asynchronous function that handles the autocomplete feature for deleting activities.
/// It takes a `Context` and a `CommandInteraction` as parameters.
/// `ctx` is the context in which this function is called.
/// `autocomplete_interaction` is the command interaction that triggered this function.
///
/// This function first gets the map of options from the command interaction.
/// It then gets the activity search string from the map of options.
/// If the activity search string is not found in the map, it defaults to a predefined string.
/// It then gets the guild ID from the command interaction.
/// If the guild ID is not found, it defaults to "0".
/// It then gets all activities by the server with the guild ID.
/// It creates a vector of activity strings from the activities.
/// It creates a vector of activity references from the activity strings.
/// It then uses `rust_fuzzy_search` to find the top matches for the activity search string in the activity references.
/// It creates a new vector for the autocomplete choices.
/// For each match, it extracts the activity name and user from the string and creates a new `AutocompleteChoice` with the name and the ID.
/// It then creates a new `CreateAutocompleteResponse` with the choices.
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
/// This function is asynchronous. It awaits the getting of all activities by the server, the fuzzy search, and the sending of the response.
pub async fn autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
    let map = get_option_map_string_autocomplete_subcommand_group(&autocomplete_interaction);
    let activity_search = map
        .get(&String::from("anime_name"))
        .unwrap_or(DEFAULT_STRING);

    let guild_id = match autocomplete_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let activities = get_data_all_activity_by_server(&guild_id).await.unwrap();
    let activity: Vec<String> = activities
        .clone()
        .into_iter()
        .map(|activity| format!("{}${}", activity.1, activity.0))
        .collect();
    let activity_refs: Vec<&str> = activity.iter().map(String::as_str).collect();

    // Use rust-fuzzy-search to find the top 5 matches
    let matches = rust_fuzzy_search::fuzzy_search_best_n(
        activity_search,
        &activity_refs,
        AUTOCOMPLETE_COUNT_LIMIT as usize,
    );

    let mut choices = Vec::new();
    for (activity, _) in matches {
        let parts: Vec<&str> = activity.split('$').collect();
        let id = parts[1].to_string();
        let name = parts[0].to_string();
        choices.push(AutocompleteChoice::new(name, id))
    }
    let data = CreateAutocompleteResponse::new().set_choices(choices);
    let builder = CreateInteractionResponse::Autocomplete(data);

    let _ = autocomplete_interaction
        .create_response(ctx.http, builder)
        .await;
}

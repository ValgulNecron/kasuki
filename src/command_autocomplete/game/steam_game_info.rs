use rust_fuzzy_search::fuzzy_search_best_n;
use serenity::all::{
    AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
    CreateInteractionResponse,
};

use crate::command_run::get_option::get_option_map_string;
use crate::constant::{APPS, AUTOCOMPLETE_COUNT_LIMIT, DEFAULT_STRING};

pub async fn autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
    let map = get_option_map_string(&autocomplete_interaction);
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

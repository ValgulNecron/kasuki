use crate::constant::{APPS, AUTOCOMPLETE_COUNT};
use rust_fuzzy_search::fuzzy_search_best_n;
use serenity::all::{
    AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
    CreateInteractionResponse,
};
use tracing::trace;

pub async fn autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
    let mut search = String::new();
    for option in &autocomplete_interaction.data.options {
        if option.name.as_str() != "type" {
            search = option.value.as_str().unwrap().to_string()
        }
    }

    let app_names: Vec<&str> = unsafe { APPS.iter().map(|app| app.0.as_str()).collect() };
    let result = fuzzy_search_best_n(search.as_str(), &app_names, AUTOCOMPLETE_COUNT as usize);
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

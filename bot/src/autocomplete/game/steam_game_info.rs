use serenity::all::{
    AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
    CreateInteractionResponse,
};
use tracing::debug;

use crate::constant::{AUTOCOMPLETE_COUNT_LIMIT, DEFAULT_STRING};
use crate::event_handler::BotData;
use crate::helper::fuzzy_search::distance_top_n;
use crate::helper::get_option::subcommand::get_option_map_string_autocomplete_subcommand;

pub async fn autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
    let map = get_option_map_string_autocomplete_subcommand(&autocomplete_interaction);
    let bot_data = ctx.data::<BotData>().clone();

    let game_search = map
        .get(&String::from("game_name"))
        .unwrap_or(DEFAULT_STRING);

    let guard = bot_data.apps.read().await;

    let app_names: Vec<String> = guard
        .clone()
        .iter()
        .map(|app| app.0.clone())
        .filter(|a| a.contains(game_search))
        .collect();

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
        .create_response(&ctx.http, builder)
        .await
    {
        debug!("Error sending response: {:?}", why);
    }
}

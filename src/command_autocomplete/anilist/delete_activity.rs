use crate::constant::AUTOCOMPLETE_COUNT;
use crate::database::general::data::get_data_all_activity_by_server;
use serenity::all::{
    AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
    CreateInteractionResponse,
};

pub async fn autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
    let mut search = String::new();
    for option in &autocomplete_interaction.data.options {
        if option.name.as_str() != "type" {
            search = option.value.as_str().unwrap().to_string()
        }
    }

    let guild_id = match autocomplete_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let activities = get_data_all_activity_by_server(&guild_id)
        .await
        .unwrap()
        .unwrap();
    let activity_strings: Vec<String> = activities
        .into_iter()
        .map(|activity| format!("{} {}", activity.1, activity.0))
        .collect();
    let activity_refs: Vec<&str> = activity_strings.iter().map(String::as_str).collect();

    // Use rust-fuzzy-search to find the top 5 matches
    let matches = rust_fuzzy_search::fuzzy_search_best_n(
        &search,
        &activity_refs,
        AUTOCOMPLETE_COUNT as usize,
    );

    let mut choices = Vec::new();
    for (activity, _) in matches {
        // Extract the activity name and user from the string
        let parts: Vec<&str> = activity.split(' ').collect();
        let id = parts[0].to_string();
        let name = parts[1].to_string();
        choices.push(AutocompleteChoice::new(name, id))
    }
    let data = CreateAutocompleteResponse::new().set_choices(choices);
    let builder = CreateInteractionResponse::Autocomplete(data);

    let _ = autocomplete_interaction
        .create_response(ctx.http, builder)
        .await;
}

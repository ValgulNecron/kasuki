use crate::sqls::general::data::get_data_all_activity_by_server;
use serenity::all::{
    AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
    CreateInteractionResponse,
};

pub async fn autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
    let mut search;
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
    let mut choices = Vec::new();
    for activity in activities {
        let user = activity.1;
        choices.push(AutocompleteChoice::new(user, activity.0))
    }
    let data = CreateAutocompleteResponse::new().set_choices(choices);
    let builder = CreateInteractionResponse::Autocomplete(data);

    let _ = autocomplete_interaction
        .create_response(ctx.http, builder)
        .await;
}

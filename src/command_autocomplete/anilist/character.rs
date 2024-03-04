use serenity::all::{
    AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
    CreateInteractionResponse,
};

use crate::anilist_struct::autocomplete::character::CharacterPageWrapper;
use crate::command_run::get_option::get_option_map_string_autocomplete_subcommand;
use crate::constant::DEFAULT_STRING;

pub async fn autocomplete(ctx: Context, autocomplete_interaction: CommandInteraction) {
    let map = get_option_map_string_autocomplete_subcommand(&autocomplete_interaction);
    let character_search = map.get(&String::from("name")).unwrap_or(DEFAULT_STRING);
    let data = CharacterPageWrapper::new_autocomplete_character(character_search).await;
    let mut choices = Vec::new();
    let character = data.data.page.characters.unwrap().clone();

    for user in character {
        let data = user.unwrap();
        let name = data.name.unwrap();
        let full = name.full.clone();
        let user_pref = name.user_preferred.clone();
        let name = user_pref.unwrap_or(full);
        choices.push(AutocompleteChoice::new(name, data.id.to_string()))
    }

    let data = CreateAutocompleteResponse::new().set_choices(choices);
    let builder = CreateInteractionResponse::Autocomplete(data);

    let _ = autocomplete_interaction
        .create_response(ctx.http, builder)
        .await;
}

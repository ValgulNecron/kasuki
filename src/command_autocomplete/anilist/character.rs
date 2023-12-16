use serenity::all::{
    AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
    CreateInteractionResponse,
};

use crate::anilist_struct::autocomplete::character::CharacterPageWrapper;

pub async fn autocomplete(ctx: Context, command: CommandInteraction) {
    let mut search = String::new();
    for option in &command.data.options {
        if option.name.as_str() != "type" {
            search = option.value.as_str().unwrap().to_string()
        }
    }
    let data = CharacterPageWrapper::new_autocomplete_character(&search.to_string()).await;
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

    let _ = command.create_response(ctx.http, builder).await;
}
